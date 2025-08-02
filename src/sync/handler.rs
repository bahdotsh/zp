use crate::config::SyncConfig;
use crate::sync::{protocol::SyncProtocol, server::SyncServer};

use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process;
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

        // Check if sync daemon is already running
        let pid_dir = env::var("HOME")
            .map(|home| PathBuf::from(home).join(".zp"))
            .unwrap_or_else(|_| PathBuf::from(".zp"));

        if !pid_dir.exists() {
            fs::create_dir_all(&pid_dir)?;
        }

        let pid_file = pid_dir.join("zp-sync-daemon.pid");

        if pid_file.exists() {
            let pid_str = fs::read_to_string(&pid_file)?;
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                #[cfg(unix)]
                {
                    let status = std::process::Command::new("kill")
                        .arg("-0")
                        .arg(pid.to_string())
                        .status();

                    if status.is_ok() && status.unwrap().success() {
                        println!("🔄 Sync daemon is already running with PID {}", pid);
                        return Ok(());
                    }
                }

                #[cfg(not(unix))]
                {
                    println!("Cannot verify if sync daemon is running, assuming it's not");
                }
            }
        }

        // Fork to background on Unix systems
        #[cfg(unix)]
        {
            use daemonize::Daemonize;
            println!("🚀 Starting sync daemon in the background");

            let pid_file = pid_dir.join("zp-sync-daemon.pid");

            // Create a new daemonize process
            let daemonize = Daemonize::new()
                .pid_file(&pid_file)
                .chown_pid_file(true)
                .working_directory("/tmp")
                .stdout(std::fs::File::create("/dev/null")?)
                .stderr(std::fs::File::create("/dev/null")?);

            match daemonize.start() {
                Ok(_) => {
                    // We're now in the daemon process
                    self.run_sync_daemon_worker().await
                }
                Err(e) => {
                    eprintln!("Error starting sync daemon: {}", e);
                    Err(e.into())
                }
            }
        }

        // For non-Unix systems, just continue execution
        #[cfg(not(unix))]
        {
            println!(
                "🚀 Starting sync daemon in the foreground (background not supported on this OS)"
            );
            return self.run_sync_daemon_worker().await;
        }
    }

    // The actual sync daemon worker process
    async fn run_sync_daemon_worker(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Get the pid file path
        let pid_dir = env::var("HOME")
            .map(|home| PathBuf::from(home).join(".zp"))
            .unwrap_or_else(|_| PathBuf::from(".zp"));

        let pid_file = pid_dir.join("zp-sync-daemon.pid");

        // Write current PID to file
        let pid = process::id();
        let mut file = File::create(&pid_file)?;
        write!(file, "{}", pid)?;

        println!("🚀 Sync daemon started with PID {}", pid);
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

    pub fn stop_sync_daemon() -> Result<(), Box<dyn std::error::Error>> {
        let pid_dir = env::var("HOME")
            .map(|home| PathBuf::from(home).join(".zp"))
            .unwrap_or_else(|_| PathBuf::from(".zp"));

        let pid_file = pid_dir.join("zp-sync-daemon.pid");

        if !pid_file.exists() {
            println!("🔄 Sync daemon is not running");
            return Ok(());
        }

        let pid_str = fs::read_to_string(&pid_file)?;
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            // Send termination signal
            #[cfg(unix)]
            {
                let status = std::process::Command::new("kill")
                    .arg(pid.to_string())
                    .status();

                if status.is_ok() && status.unwrap().success() {
                    println!("🛑 Stopped sync daemon with PID {}", pid);
                    // Remove PID file
                    fs::remove_file(&pid_file)?;
                } else {
                    println!("❌ Failed to stop sync daemon with PID {}", pid);
                }
            }

            // For Windows
            #[cfg(windows)]
            {
                use std::process::Command;
                let status = Command::new("taskkill")
                    .args(&["/PID", &pid.to_string(), "/F"])
                    .status();

                if status.is_ok() && status.unwrap().success() {
                    println!("🛑 Stopped sync daemon with PID {}", pid);
                    // Remove PID file
                    fs::remove_file(&pid_file)?;
                } else {
                    println!("❌ Failed to stop sync daemon with PID {}", pid);
                }
            }
        } else {
            println!("❌ Invalid PID in sync daemon file");
        }

        Ok(())
    }

    pub fn sync_daemon_status() -> Result<(), Box<dyn std::error::Error>> {
        let pid_dir = env::var("HOME")
            .map(|home| PathBuf::from(home).join(".zp"))
            .unwrap_or_else(|_| PathBuf::from(".zp"));

        let pid_file = pid_dir.join("zp-sync-daemon.pid");

        if !pid_file.exists() {
            println!("🔄 Sync daemon is not running");
            return Ok(());
        }

        let pid_str = fs::read_to_string(&pid_file)?;
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            // Check if process is running
            #[cfg(unix)]
            {
                let status = std::process::Command::new("kill")
                    .arg("-0")
                    .arg(pid.to_string())
                    .status();

                if status.is_ok() && status.unwrap().success() {
                    println!("🔄 Sync daemon is running with PID {}", pid);
                } else {
                    println!("🔄 Sync daemon is not running (stale PID file)");
                    // Remove stale PID file
                    fs::remove_file(&pid_file)?;
                }
            }

            // For Windows or fallback
            #[cfg(not(unix))]
            {
                println!("🔄 Sync daemon appears to be running with PID {}", pid);
            }
        } else {
            println!("❌ Invalid PID in sync daemon file");
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
