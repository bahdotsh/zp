use crate::config::SyncConfig;
use crate::sync::{protocol::SyncProtocol, server::SyncServer};

use tokio::time::{interval, Duration};

pub struct SyncHandler {
    config: SyncConfig,
}

impl SyncHandler {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = SyncConfig::load()?;
        Ok(Self { config })
    }

    pub async fn start_daemon(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.enabled {
            println!("🔕 Sync is disabled in configuration");
            return Ok(());
        }

        println!("🚀 Starting zp sync daemon...");
        println!("📍 Peer ID: {}", self.config.peer_id);
        println!("🔌 Listening on port: {}", self.config.listen_port);

        // Create sync server and protocol handler
        let server = SyncServer::new(self.config.clone());
        let mut protocol = SyncProtocol::new(self.config.clone());

        // Start HTTP server in background
        let server_handle = tokio::spawn(async move {
            if let Err(e) = server.start().await {
                eprintln!("❌ Sync server error: {}", e);
            }
        });

        // Start periodic sync if enabled
        let sync_handle = if self.config.auto_sync {
            let mut interval = interval(Duration::from_secs(self.config.sync_interval_seconds));
            tokio::spawn(async move {
                loop {
                    interval.tick().await;
                    if let Err(e) = protocol.sync_once().await {
                        eprintln!("❌ Sync error: {}", e);
                    }

                    // Small delay to prevent overwhelming the system
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            })
        } else {
            tokio::spawn(async {
                // Just wait indefinitely if auto-sync is disabled
                loop {
                    tokio::time::sleep(Duration::from_secs(3600)).await;
                }
            })
        };

        // Wait for either task to complete (they shouldn't under normal circumstances)
        tokio::select! {
            _ = server_handle => {
                println!("🔄 Sync server stopped");
            }
            _ = sync_handle => {
                println!("🔄 Sync handler stopped");
            }
        }

        Ok(())
    }

    pub async fn sync_now(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.enabled {
            println!("🔕 Sync is disabled in configuration");
            return Ok(());
        }

        let mut protocol = SyncProtocol::new(self.config.clone());
        protocol.sync_once().await
    }

    pub fn show_config(&self) {
        println!("📋 Sync Configuration:");
        println!("  Enabled: {}", self.config.enabled);
        println!("  Peer ID: {}", self.config.peer_id);
        println!("  Listen Port: {}", self.config.listen_port);
        println!("  Auto Sync: {}", self.config.auto_sync);
        println!("  Sync Interval: {}s", self.config.sync_interval_seconds);
        println!("  Configured Peers: {}", self.config.peers.len());

        for (peer_id, peer_config) in &self.config.peers {
            let status = if peer_config.enabled { "✅" } else { "❌" };
            let conn_type = if peer_config.ssh_config.is_some() {
                "SSH"
            } else {
                "HTTP"
            };
            println!(
                "    {} {} [{}]: {}",
                status, peer_id, conn_type, peer_config.endpoint
            );
        }
    }

    pub fn enable_sync(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.config.enabled = true;
        self.config.save()?;
        println!("✅ Sync enabled");
        Ok(())
    }

    pub fn disable_sync(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.config.enabled = false;
        self.config.save()?;
        println!("🔕 Sync disabled");
        Ok(())
    }

    pub fn add_peer(
        &mut self,
        peer_id: String,
        endpoint: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.config.add_peer(peer_id.clone(), endpoint.clone());
        self.config.save()?;
        println!("➕ Added peer: {} -> {}", peer_id, endpoint);
        Ok(())
    }

    pub fn remove_peer(&mut self, peer_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.peers.remove(peer_id).is_some() {
            self.config.save()?;
            println!("➖ Removed peer: {}", peer_id);
        } else {
            println!("❌ Peer not found: {}", peer_id);
        }
        Ok(())
    }

    pub fn set_peer_enabled(
        &mut self,
        peer_id: &str,
        enabled: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(peer) = self.config.peers.get_mut(peer_id) {
            peer.enabled = enabled;
            self.config.save()?;
            let status = if enabled { "enabled" } else { "disabled" };
            println!("🔄 Peer {} {}", peer_id, status);
        } else {
            println!("❌ Peer not found: {}", peer_id);
        }
        Ok(())
    }

    pub async fn test_peer_connection(
        &self,
        peer_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(peer_config) = self.config.peers.get(peer_id) {
            let protocol = SyncProtocol::new(self.config.clone());
            let endpoint = protocol.resolve_endpoint_public(peer_config).await?;

            println!("🔍 Testing connection to {} at {}", peer_id, endpoint);

            let client = reqwest::Client::new();
            let response = client
                .get(format!("{}/health", endpoint))
                .timeout(Duration::from_secs(5))
                .send()
                .await?;

            if response.status().is_success() {
                println!("✅ Connection successful to {}", peer_id);
            } else {
                println!(
                    "❌ Connection failed to {} (status: {})",
                    peer_id,
                    response.status()
                );
            }
        } else {
            println!("❌ Peer not found: {}", peer_id);
        }

        Ok(())
    }
}

impl Default for SyncHandler {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            config: SyncConfig::default(),
        })
    }
}
