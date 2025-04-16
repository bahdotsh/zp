mod engine;
mod handler;

pub use self::engine::{SyncEngine, start_sync_engine};
pub use self::handler::SyncHandler;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Status of a clipboard entry in terms of synchronization
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SyncStatus {
    LocalOnly,
    Synced,
    Conflicted,
}

impl Default for SyncStatus {
    fn default() -> Self {
        SyncStatus::LocalOnly
    }
}

/// Strategies for resolving conflicts between clipboard entries
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ConflictResolutionStrategy {
    KeepNewest,
    KeepBoth,
    PreferLocalDevice,
    PreferSpecificDevice(String),
}

/// Configuration for the synchronization feature
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncConfig {
    pub enabled: bool,
    pub sync_dir: PathBuf,
    pub device_name: String,
    pub auto_merge: bool,
    pub conflict_resolution: ConflictResolutionStrategy,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sync_dir: PathBuf::from(dirs::home_dir().unwrap_or_default().join(".zp/sync")),
            device_name: hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown-device".to_string()),
            auto_merge: true,
            conflict_resolution: ConflictResolutionStrategy::KeepNewest,
        }
    }
}

/// Loads sync configuration from file or creates default
pub fn load_sync_config() -> Result<SyncConfig, std::io::Error> {
    let config_path = get_sync_config_path();
    
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    } else {
        let config = SyncConfig::default();
        save_sync_config(&config)?;
        Ok(config)
    }
}

/// Saves sync configuration to file
pub fn save_sync_config(config: &SyncConfig) -> Result<(), std::io::Error> {
    let config_path = get_sync_config_path();
    
    // Ensure the directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let serialized = serde_json::to_string_pretty(config)?;
    std::fs::write(config_path, serialized)
}

/// Returns the path to the sync configuration file
fn get_sync_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".zp/sync_config.json")
}

/// Generates a unique ID for clipboard entries
pub fn generate_entry_id() -> String {
    Uuid::new_v4().to_string()
}

/// Gets the device ID for this machine
pub fn get_device_id() -> String {
    // Try to load or create a persistent device ID
    let device_id_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".zp/device_id");
    
    if device_id_path.exists() {
        match std::fs::read_to_string(&device_id_path) {
            Ok(id) if !id.is_empty() => return id.trim().to_string(),
            _ => {}
        }
    }
    
    // Generate a new device ID if one doesn't exist
    let device_id = Uuid::new_v4().to_string();
    
    // Ensure the directory exists
    if let Some(parent) = device_id_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    
    // Save the device ID for future use
    let _ = std::fs::write(&device_id_path, &device_id);
    
    device_id
} 