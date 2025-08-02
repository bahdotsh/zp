use crate::config::{PeerConfig, SyncConfig};
use crate::history::{load_clipboard_history, ClipboardHistoryEntry};
use crate::sync::{create_sync_message, SyncData, SyncManager, SyncMessage, SyncMessageType};

use std::process::Command;
use std::time::Duration;

pub struct SyncProtocol {
    manager: SyncManager,
}

impl SyncProtocol {
    pub fn new(config: SyncConfig) -> Self {
        Self {
            manager: SyncManager::new(config),
        }
    }

    pub async fn sync_with_peers(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.manager.is_enabled() {
            return Ok(());
        }

        println!("🔄 Starting sync with peers...");

        let peers: Vec<_> = self
            .manager
            .get_enabled_peers()
            .into_iter()
            .map(|(id, config)| (id.clone(), config.clone()))
            .collect();
        for (peer_id, peer_config) in peers {
            if let Err(e) = self.sync_with_peer(&peer_id, &peer_config).await {
                eprintln!("❌ Failed to sync with peer {}: {}", peer_id, e);
            }
        }

        Ok(())
    }

    async fn sync_with_peer(
        &mut self,
        peer_id: &str,
        peer_config: &PeerConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let endpoint = self.resolve_endpoint(peer_config).await?;

        println!("🔗 Syncing with peer {} at {}", peer_id, endpoint);

        // First, perform handshake
        self.handshake(&endpoint).await?;

        // Get last sync timestamp for this peer
        let last_sync = self.manager.get_last_sync(peer_id).unwrap_or(0);

        // Request history since last sync
        let remote_entries = self.request_history(&endpoint, last_sync).await?;

        if !remote_entries.is_empty() {
            println!(
                "📥 Received {} new entries from {}",
                remote_entries.len(),
                peer_id
            );
            self.merge_remote_entries(remote_entries).await?;
        }

        // Send our local entries that are newer than peer's last sync
        let local_entries = self.get_local_entries_since(last_sync)?;
        if !local_entries.is_empty() {
            println!("📤 Sending {} entries to {}", local_entries.len(), peer_id);
            self.send_entries(&endpoint, local_entries).await?;
        }

        // Update last sync timestamp
        let current_timestamp = chrono::Utc::now().timestamp();
        self.manager
            .update_last_sync(peer_id.to_string(), current_timestamp);

        println!("✅ Sync completed with peer {}", peer_id);
        Ok(())
    }

    async fn resolve_endpoint(
        &self,
        peer_config: &PeerConfig,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(ssh_config) = &peer_config.ssh_config {
            // Set up SSH tunnel if needed
            self.setup_ssh_tunnel(ssh_config).await?;
            Ok(format!("http://localhost:{}", ssh_config.tunnel_local_port))
        } else {
            Ok(peer_config.endpoint.clone())
        }
    }

    async fn setup_ssh_tunnel(
        &self,
        ssh_config: &crate::config::SshConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check if tunnel is already running
        if self.is_port_in_use(ssh_config.tunnel_local_port) {
            return Ok(()); // Tunnel already exists
        }

        let mut ssh_cmd = Command::new("ssh");
        ssh_cmd
            .arg("-N") // Don't execute remote command
            .arg("-L") // Local port forwarding
            .arg(format!(
                "{}:localhost:{}",
                ssh_config.tunnel_local_port, ssh_config.remote_port
            ))
            .arg("-p")
            .arg(ssh_config.ssh_port.unwrap_or(22).to_string())
            .arg(format!("{}@{}", ssh_config.ssh_user, ssh_config.ssh_host));

        if let Some(identity_file) = &ssh_config.identity_file {
            ssh_cmd.arg("-i").arg(identity_file);
        }

        // Start SSH tunnel in background
        ssh_cmd.spawn()?;

        // Wait a moment for tunnel to establish
        tokio::time::sleep(Duration::from_millis(1000)).await;

        println!(
            "🔐 SSH tunnel established on port {}",
            ssh_config.tunnel_local_port
        );
        Ok(())
    }

    fn is_port_in_use(&self, port: u16) -> bool {
        std::net::TcpListener::bind(format!("127.0.0.1:{}", port)).is_err()
    }

    async fn handshake(&self, endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let handshake_msg = create_sync_message(
            SyncMessageType::Handshake,
            self.manager.get_peer_id().to_string(),
            None,
        );

        let response = client
            .post(format!("{}/sync", endpoint))
            .json(&handshake_msg)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;

        if response.status().is_success() {
            let _response_msg: SyncMessage = response.json().await?;
            Ok(())
        } else {
            Err(format!("Handshake failed with status: {}", response.status()).into())
        }
    }

    async fn request_history(
        &self,
        endpoint: &str,
        since_timestamp: i64,
    ) -> Result<Vec<ClipboardHistoryEntry>, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let response = client
            .get(format!("{}/history", endpoint))
            .query(&[("since", since_timestamp.to_string())])
            .timeout(Duration::from_secs(30))
            .send()
            .await?;

        if response.status().is_success() {
            let entries: Vec<ClipboardHistoryEntry> = response.json().await?;
            Ok(entries)
        } else {
            Err(format!("Failed to request history: {}", response.status()).into())
        }
    }

    async fn send_entries(
        &self,
        endpoint: &str,
        entries: Vec<ClipboardHistoryEntry>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let sync_msg = create_sync_message(
            SyncMessageType::ClipboardSync,
            self.manager.get_peer_id().to_string(),
            Some(SyncData::ClipboardEntries(entries)),
        );

        let response = client
            .post(format!("{}/sync", endpoint))
            .json(&sync_msg)
            .timeout(Duration::from_secs(30))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to send entries: {}", response.status()).into())
        }
    }

    fn get_local_entries_since(
        &self,
        since_timestamp: i64,
    ) -> Result<Vec<ClipboardHistoryEntry>, Box<dyn std::error::Error>> {
        let entries = load_clipboard_history()?;

        let filtered_entries = entries
            .into_iter()
            .filter(|entry| {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp) {
                    dt.timestamp() > since_timestamp
                } else {
                    false
                }
            })
            .collect();

        Ok(filtered_entries)
    }

    async fn merge_remote_entries(
        &self,
        remote_entries: Vec<ClipboardHistoryEntry>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Load local history
        let local_entries = load_clipboard_history().unwrap_or_else(|_| vec![]);

        // Create a set of existing content to avoid duplicates
        let existing_content: std::collections::HashSet<String> = local_entries
            .iter()
            .map(|entry| entry.content.clone())
            .collect();

        // Filter out entries that already exist locally
        let new_entries: Vec<ClipboardHistoryEntry> = remote_entries
            .into_iter()
            .filter(|entry| !existing_content.contains(&entry.content))
            .collect();

        if !new_entries.is_empty() {
            // Combine and sort all entries by timestamp
            let mut all_entries = local_entries;
            all_entries.extend(new_entries.clone());

            all_entries.sort_by(|a, b| {
                let ts_a = chrono::DateTime::parse_from_rfc3339(&a.timestamp).unwrap_or_default();
                let ts_b = chrono::DateTime::parse_from_rfc3339(&b.timestamp).unwrap_or_default();
                ts_a.cmp(&ts_b)
            });

            let num_new = new_entries.len();
            // Save merged history
            crate::sync::server::save_merged_history(all_entries)?;
            println!("📋 Merged {} new clipboard entries", num_new);
        }

        Ok(())
    }

    pub async fn sync_once(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.sync_with_peers().await
    }

    pub async fn resolve_endpoint_public(
        &self,
        peer_config: &PeerConfig,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(ssh_config) = &peer_config.ssh_config {
            // Set up SSH tunnel if needed
            self.setup_ssh_tunnel(ssh_config).await?;
            Ok(format!("http://localhost:{}", ssh_config.tunnel_local_port))
        } else {
            Ok(peer_config.endpoint.clone())
        }
    }
}
