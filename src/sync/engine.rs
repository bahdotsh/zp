use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use crate::sync::{SyncConfig, SyncHandler};
use std::io;

/// SyncEngine is responsible for the file-based synchronization functionality
pub struct SyncEngine {
    config: SyncConfig,
    _handler: Arc<Mutex<Option<SyncHandler>>>,
    running: bool,
}

impl SyncEngine {
    /// Create a new sync engine with the given configuration
    pub async fn new(config: SyncConfig) -> Result<Self, io::Error> {
        // Create sync directory if it doesn't exist
        if !config.sync_dir.exists() {
            std::fs::create_dir_all(&config.sync_dir)?;
        }

        Ok(Self {
            config,
            _handler: Arc::new(Mutex::new(None)),
            running: false,
        })
    }

    /// Start the sync engine and begin syncing
    pub async fn start(&mut self) -> Result<(), io::Error> {
        if self.running {
            return Ok(());
        }

        println!("Starting sync engine with device name: {}", self.config.device_name);
        println!("Sync directory: {:?}", self.config.sync_dir);

        // Initialize the sync handler
        let mut handler = SyncHandler::new(self.config.sync_dir.clone())?;
        handler.start().await?;
        
        // Store the handler
        let mut locked_handler = self._handler.lock().await;
        *locked_handler = Some(handler);

        // Start a background task to monitor the sync status
        let config = self.config.clone();
        let handler_clone = self._handler.clone();
        
        tokio::spawn(async move {
            Self::monitor_sync_task(handler_clone, config).await;
        });

        self.running = true;
        Ok(())
    }

    /// Stop the sync engine
    pub async fn stop(&mut self) -> Result<(), io::Error> {
        if !self.running {
            return Ok(());
        }

        println!("Stopping sync engine");
        
        // Stop the sync handler
        let mut locked_handler = self._handler.lock().await;
        if let Some(handler) = locked_handler.as_mut() {
            handler.stop();
        }
        *locked_handler = None;

        self.running = false;
        Ok(())
    }

    /// Background task to monitor sync status
    async fn monitor_sync_task(
        _handler: Arc<Mutex<Option<SyncHandler>>>,
        config: SyncConfig,
    ) {
        // Create a heartbeat file to show this device is active
        let heartbeat_file = config.sync_dir.join(format!("device_{}_heartbeat.json", config.device_name));
        
        loop {
            // Update heartbeat file with current timestamp to indicate active device
            let heartbeat_data = serde_json::json!({
                "device_name": config.device_name,
                "last_seen": chrono::Local::now().to_rfc3339(),
                "device_id": crate::sync::get_device_id(),
            });
            
            if let Err(e) = std::fs::write(
                &heartbeat_file,
                serde_json::to_string_pretty(&heartbeat_data).unwrap(),
            ) {
                eprintln!("Failed to write heartbeat file: {}", e);
            }
            
            // Check for other active devices
            if let Ok(entries) = std::fs::read_dir(&config.sync_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(file_name) = path.file_name() {
                        if let Some(name) = file_name.to_str() {
                            if name.starts_with("device_") && name.ends_with("_heartbeat.json") 
                                && !name.contains(&config.device_name) {
                                // Found another device's heartbeat file
                                if let Ok(content) = std::fs::read_to_string(&path) {
                                    if let Ok(heartbeat) = serde_json::from_str::<serde_json::Value>(&content) {
                                        if let Some(last_seen) = heartbeat["last_seen"].as_str() {
                                            if let Some(device_name) = heartbeat["device_name"].as_str() {
                                                println!("Detected peer device: {} (last seen: {})", device_name, last_seen);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            sleep(Duration::from_secs(30)).await;
        }
    }
}

/// Convenience function to start the sync engine with defaults
pub async fn start_sync_engine() -> Result<Arc<Mutex<SyncEngine>>, io::Error> {
    let config = crate::sync::load_sync_config()?;
    
    if !config.enabled {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Sync is disabled in configuration",
        ));
    }
    
    let mut engine = SyncEngine::new(config).await?;
    engine.start().await?;
    
    Ok(Arc::new(Mutex::new(engine)))
} 