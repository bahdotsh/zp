use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncConfig {
    pub enabled: bool,
    pub peer_id: String,
    pub listen_port: u16,
    pub peers: HashMap<String, PeerConfig>,
    pub sync_interval_seconds: u64,
    pub auto_sync: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeerConfig {
    pub endpoint: String, // "http://192.168.1.100:8080" or "ssh://user@host:port"
    pub enabled: bool,
    pub ssh_config: Option<SshConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SshConfig {
    pub tunnel_local_port: u16, // Local port for SSH tunnel
    pub remote_port: u16,       // Remote port where sync service runs
    pub ssh_user: String,
    pub ssh_host: String,
    pub ssh_port: Option<u16>,         // Default to 22
    pub identity_file: Option<String>, // SSH key path
}

impl Default for SyncConfig {
    fn default() -> Self {
        let hostname = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let username = env::var("USER").unwrap_or_else(|_| "user".to_string());

        // Generate a simple random suffix
        let random_suffix: String = (0..4)
            .map(|_| {
                let chars = b"abcdefghijklmnopqrstuvwxyz0123456789";
                chars[fastrand::usize(..chars.len())] as char
            })
            .collect();

        Self {
            enabled: false,
            peer_id: format!("{}@{}-{}", username, hostname, random_suffix),
            listen_port: 8080,
            peers: HashMap::new(),
            sync_interval_seconds: 30,
            auto_sync: true,
        }
    }
}

impl SyncConfig {
    pub fn config_dir() -> PathBuf {
        env::var("HOME")
            .map(|home| PathBuf::from(home).join(".zp"))
            .unwrap_or_else(|_| PathBuf::from(".zp"))
    }

    pub fn config_file() -> PathBuf {
        Self::config_dir().join("sync_config.json")
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_file = Self::config_file();

        if !config_file.exists() {
            // Create default config if it doesn't exist
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_file)?;

        // Try to parse as the current format first
        if let Ok(config) = serde_json::from_str::<SyncConfig>(&content) {
            return Ok(config);
        }

        // If that fails, try to migrate from older format
        match Self::migrate_from_old_format(&content) {
            Ok(config) => {
                // Save the migrated config
                config.save()?;
                println!("📋 Migrated sync configuration to new format");
                Ok(config)
            }
            Err(_) => {
                // If migration fails, create a new default config
                println!("⚠️  Could not migrate old sync config, creating new one");
                let default_config = Self::default();
                default_config.save()?;
                Ok(default_config)
            }
        }
    }

    fn migrate_from_old_format(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Try to parse as a generic JSON value first
        let json: serde_json::Value = serde_json::from_str(content)?;

        let mut config = Self::default();

        // Migrate known fields
        if let Some(enabled) = json.get("enabled").and_then(|v| v.as_bool()) {
            config.enabled = enabled;
        }

        if let Some(port) = json.get("listen_port").and_then(|v| v.as_u64()) {
            config.listen_port = port as u16;
        }

        if let Some(interval) = json.get("sync_interval_seconds").and_then(|v| v.as_u64()) {
            config.sync_interval_seconds = interval;
        }

        if let Some(auto_sync) = json.get("auto_sync").and_then(|v| v.as_bool()) {
            config.auto_sync = auto_sync;
        }

        // Try to migrate peers if they exist
        if let Some(peers_obj) = json.get("peers").and_then(|v| v.as_object()) {
            for (peer_id, peer_data) in peers_obj {
                if let Ok(peer_config) = serde_json::from_value::<PeerConfig>(peer_data.clone()) {
                    config.peers.insert(peer_id.clone(), peer_config);
                }
            }
        }

        // peer_id will use the default generated value

        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_dir = Self::config_dir();
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        let config_file = Self::config_file();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_file, content)?;
        Ok(())
    }

    pub fn add_peer(&mut self, peer_id: String, endpoint: String) {
        let peer_config = if endpoint.starts_with("ssh://") {
            // Parse SSH endpoint: ssh://user@host:port
            let ssh_part = endpoint.strip_prefix("ssh://").unwrap();
            let (user_host, port) = if let Some((uh, p)) = ssh_part.split_once(':') {
                (uh, p.parse().unwrap_or(22))
            } else {
                (ssh_part, 22)
            };

            let (user, host) = if let Some((u, h)) = user_host.split_once('@') {
                (u.to_string(), h.to_string())
            } else {
                (
                    env::var("USER").unwrap_or_else(|_| "user".to_string()),
                    user_host.to_string(),
                )
            };

            PeerConfig {
                endpoint: endpoint.clone(),
                enabled: true,
                ssh_config: Some(SshConfig {
                    tunnel_local_port: self.find_available_port(),
                    remote_port: 8080, // Default remote sync port
                    ssh_user: user,
                    ssh_host: host,
                    ssh_port: Some(port),
                    identity_file: None,
                }),
            }
        } else {
            PeerConfig {
                endpoint,
                enabled: true,
                ssh_config: None,
            }
        };

        self.peers.insert(peer_id, peer_config);
    }

    fn find_available_port(&self) -> u16 {
        // Simple port allocation starting from 8081
        let mut port = 8081;
        let used_ports: Vec<u16> = self
            .peers
            .values()
            .filter_map(|p| p.ssh_config.as_ref().map(|ssh| ssh.tunnel_local_port))
            .collect();

        while used_ports.contains(&port) {
            port += 1;
        }
        port
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = SyncConfig::default();
        assert!(!config.enabled);
        assert!(config.peer_id.contains('@'));
        assert_eq!(config.listen_port, 8080);
    }

    #[test]
    fn test_add_http_peer() {
        let mut config = SyncConfig::default();
        config.add_peer(
            "test-peer".to_string(),
            "http://192.168.1.100:8080".to_string(),
        );

        assert!(config.peers.contains_key("test-peer"));
        let peer = &config.peers["test-peer"];
        assert!(peer.enabled);
        assert!(peer.ssh_config.is_none());
    }

    #[test]
    fn test_add_ssh_peer() {
        let mut config = SyncConfig::default();
        config.add_peer(
            "ssh-peer".to_string(),
            "ssh://user@remote.host:22".to_string(),
        );

        assert!(config.peers.contains_key("ssh-peer"));
        let peer = &config.peers["ssh-peer"];
        assert!(peer.enabled);
        assert!(peer.ssh_config.is_some());

        let ssh_config = peer.ssh_config.as_ref().unwrap();
        assert_eq!(ssh_config.ssh_user, "user");
        assert_eq!(ssh_config.ssh_host, "remote.host");
        assert_eq!(ssh_config.ssh_port, Some(22));
    }
}
