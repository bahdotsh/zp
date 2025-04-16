use std::path::{Path, PathBuf};
use std::io;
use crate::history::ClipboardHistoryEntry;
use std::fs;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event};
use std::time::Duration;
use tokio::time::sleep;

/// Handles the synchronization of clipboard history between devices
pub struct SyncHandler {
    sync_dir: PathBuf,
    _device_id: String,
    watcher: Option<RecommendedWatcher>,
    running: bool,
}

impl SyncHandler {
    /// Create a new sync handler
    pub fn new(sync_dir: PathBuf) -> Result<Self, io::Error> {
        Ok(Self {
            sync_dir,
            _device_id: crate::sync::get_device_id(),
            watcher: None,
            running: false,
        })
    }
    
    /// Start the sync handler
    pub async fn start(&mut self) -> Result<(), io::Error> {
        if self.running {
            return Ok(());
        }
        
        // Ensure the sync directory exists
        if !self.sync_dir.exists() {
            fs::create_dir_all(&self.sync_dir)?;
        }
        
        // Create the watcher
        let (tx, rx) = std::sync::mpsc::channel();
        
        // Create a file system watcher - convert notify errors to io::Error
        let watcher = match notify::recommended_watcher(tx) {
            Ok(w) => w,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
        };
        
        // Store the watcher
        self.watcher = Some(watcher);
        
        // Watch the sync directory - must happen after storing the watcher
        if let Some(watcher) = &mut self.watcher {
            if let Err(e) = watcher.watch(&self.sync_dir, RecursiveMode::Recursive) {
                return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
            }
        }
        
        self.running = true;
        
        // Start the file system event loop
        let sync_dir = self.sync_dir.clone();
        
        tokio::spawn(async move {
            loop {
                match rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(event) => {
                        match event {
                            Ok(event) => Self::handle_file_event(event, &sync_dir).await,
                            Err(e) => eprintln!("Watch error: {:?}", e),
                        }
                    },
                    Err(_) => {
                        // Timeout, continue
                    }
                }
                
                // Sleep a bit to prevent high CPU usage
                sleep(Duration::from_millis(100)).await;
            }
        });
        
        Ok(())
    }
    
    /// Stop the sync handler
    pub fn stop(&mut self) {
        self.watcher = None;
        self.running = false;
    }
    
    /// Handle a file system event
    async fn handle_file_event(event: Event, _sync_dir: &Path) {
        // Check if it's a relevant file event
        if let Some(path) = event.paths.first() {
            if path.file_name().map_or(false, |name| name == "clipboard_history_sync.json") {
                if event.kind.is_modify() || event.kind.is_create() {
                    Self::import_from_sync_file(path).await.unwrap_or_else(|e| {
                        eprintln!("Failed to import from sync file: {}", e);
                    });
                }
            }
        }
    }
    
    /// Import entries from a sync file
    async fn import_from_sync_file(path: &Path) -> Result<(), io::Error> {
        // Read the sync file
        let content = fs::read_to_string(path)?;
        let sync_entries: Vec<ClipboardHistoryEntry> = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        
        if sync_entries.is_empty() {
            return Ok(());
        }
        
        // Get local entries
        let local_path = dirs::home_dir()
            .unwrap_or_default()
            .join(".zp/clipboard_history.json");
            
        let local_entries = if local_path.exists() {
            let content = fs::read_to_string(&local_path)?;
            serde_json::from_str::<Vec<ClipboardHistoryEntry>>(&content)
                .unwrap_or_else(|_| vec![])
        } else {
            vec![]
        };
        
        // Merge entries
        let merged_entries = Self::merge_entries(local_entries, sync_entries);
        
        // Save merged entries
        let serialized = serde_json::to_string_pretty(&merged_entries)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        fs::write(local_path, serialized)?;
        
        Ok(())
    }
    
    /// Merge local and synced entries, handling conflicts
    fn merge_entries(
        local_entries: Vec<ClipboardHistoryEntry>, 
        sync_entries: Vec<ClipboardHistoryEntry>
    ) -> Vec<ClipboardHistoryEntry> {
        let mut merged = local_entries;
        let device_id = crate::sync::get_device_id();
        
        // Load conflict resolution strategy
        let config = crate::sync::load_sync_config().unwrap_or_default();
        
        for sync_entry in sync_entries {
            // Skip entries that came from this device
            if sync_entry.device_id == device_id {
                continue;
            }
            
            // Check if entry with same ID already exists
            let existing_index = merged.iter().position(|e| e.entry_id == sync_entry.entry_id);
            
            if let Some(index) = existing_index {
                // Handle conflict based on strategy
                match config.conflict_resolution {
                    crate::sync::ConflictResolutionStrategy::KeepNewest => {
                        // Compare timestamps
                        if sync_entry.timestamp > merged[index].timestamp {
                            merged[index] = sync_entry;
                        }
                    },
                    crate::sync::ConflictResolutionStrategy::KeepBoth => {
                        // Keep both by adding the sync entry in addition to the existing one
                        merged.push(sync_entry);
                    },
                    crate::sync::ConflictResolutionStrategy::PreferLocalDevice => {
                        // Do nothing, keep the local entry
                    },
                    crate::sync::ConflictResolutionStrategy::PreferSpecificDevice(ref device) => {
                        // Replace if entry is from the preferred device
                        if sync_entry.device_id == *device {
                            merged[index] = sync_entry;
                        }
                    },
                }
            } else {
                // No conflict, add the new entry
                merged.push(sync_entry);
            }
        }
        
        // Sort by timestamp (newest first)
        merged.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        merged
    }
} 