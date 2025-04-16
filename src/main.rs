use clap::Parser;
use std::env;
use std::process;
use zp::history::print_clipboard_history;
use zp::sync::{load_sync_config, save_sync_config};
use zp::{daemon_status, start_daemon, stop_daemon, Query, Zp};
use std::path::PathBuf;

fn main() {
    // Parse command-line arguments into a Zp struct
    let zp = Zp::parse();

    // Special hidden flag for daemon worker process
    if env::args().any(|arg| arg == "--daemon-worker") {
        if let Err(e) = zp::run_daemon_worker() {
            eprintln!("Daemon worker failed: {}", e);
            process::exit(1);
        }
        return;
    }

    // Handle sync configuration options
    if zp.sync_enable || zp.sync_disable || zp.sync_dir.is_some() || zp.sync_device_name.is_some() {
        handle_sync_config(&zp);
        return;
    }

    // Show sync status
    if zp.sync_status {
        show_sync_status();
        return;
    }

    // Check daemon commands
    if zp.daemon {
        if let Err(e) = start_daemon() {
            eprintln!("Failed to start daemon: {}", e);
            process::exit(1);
        }
        return;
    }

    if zp.stop_daemon {
        if let Err(e) = stop_daemon() {
            eprintln!("Failed to stop daemon: {}", e);
            process::exit(1);
        }
        return;
    }

    if zp.daemon_status {
        if let Err(e) = daemon_status() {
            eprintln!("Failed to check daemon status: {}", e);
            process::exit(1);
        }
        return;
    }

    // Original logic for logs and other commands
    if zp.logs {
        print_clipboard_history().unwrap();
    } else {
        match Query::build(&zp) {
            Ok(_) => {
                // Run the application with the Zp struct (passing the original parsed Zp)
                if let Err(e) = zp::run(zp) {
                    eprintln!("Application error: {e}");
                    process::exit(1);
                }
            }
            Err(err) => {
                eprintln!("Problem parsing query: {}", err);
                process::exit(1);
            }
        }
    }
}

// Handle sync configuration changes
fn handle_sync_config(zp: &Zp) {
    match load_sync_config() {
        Ok(mut config) => {
            let mut changed = false;

            // Update enabled status
            if zp.sync_enable {
                config.enabled = true;
                changed = true;
                println!("Clipboard history syncing enabled");
            } else if zp.sync_disable {
                config.enabled = false;
                changed = true;
                println!("Clipboard history syncing disabled");
            }

            // Update sync directory if specified
            if let Some(dir) = &zp.sync_dir {
                let path = PathBuf::from(dir);
                config.sync_dir = path;
                changed = true;
                println!("Sync directory set to: {}", dir);
            }

            // Update device name if specified
            if let Some(name) = &zp.sync_device_name {
                config.device_name = name.clone();
                changed = true;
                println!("Device name set to: {}", name);
            }

            // Save the updated configuration
            if changed {
                if let Err(e) = save_sync_config(&config) {
                    eprintln!("Failed to save sync configuration: {}", e);
                    process::exit(1);
                }
                
                println!("Sync configuration updated. Restart the daemon for changes to take effect.");
                println!("Run: zp --stop-daemon && zp --daemon");
            }
        }
        Err(e) => {
            eprintln!("Failed to load sync configuration: {}", e);
            process::exit(1);
        }
    }
}

// Show sync status
fn show_sync_status() {
    match load_sync_config() {
        Ok(config) => {
            println!("Clipboard History Sync Status:");
            println!("----------------------------");
            println!("Enabled: {}", if config.enabled { "Yes" } else { "No" });
            println!("Device Name: {}", config.device_name);
            println!("Sync Directory: {:?}", config.sync_dir);
            println!("Auto Merge: {}", if config.auto_merge { "Yes" } else { "No" });
            
            let resolution_str = match &config.conflict_resolution {
                zp::sync::ConflictResolutionStrategy::KeepNewest => "Keep Newest".to_string(),
                zp::sync::ConflictResolutionStrategy::KeepBoth => "Keep Both".to_string(),
                zp::sync::ConflictResolutionStrategy::PreferLocalDevice => "Prefer Local Device".to_string(),
                zp::sync::ConflictResolutionStrategy::PreferSpecificDevice(ref device) => {
                    format!("Prefer Device: {}", device)
                },
            };
            println!("Conflict Resolution: {}", resolution_str);
        }
        Err(e) => {
            eprintln!("Failed to load sync configuration: {}", e);
            process::exit(1);
        }
    }
}
