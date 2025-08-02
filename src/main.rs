use clap::Parser;
use std::env;
use std::process;
use zp::history::print_clipboard_history;
use zp::sync::handler::SyncHandler;
use zp::{daemon_status, start_daemon, stop_daemon, Query, Zp};

#[tokio::main]
async fn main() {
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

    // Check daemon commands first
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

    // Handle sync commands
    if zp.sync_daemon {
        handle_sync_daemon().await;
        return;
    }

    if zp.sync_now {
        handle_sync_now().await;
        return;
    }

    if zp.sync_config {
        handle_sync_config();
        return;
    }

    if zp.sync_enable {
        handle_sync_enable();
        return;
    }

    if zp.sync_disable {
        handle_sync_disable();
        return;
    }

    if let Some(peer_info) = &zp.add_peer {
        handle_add_peer(peer_info);
        return;
    }

    if let Some(peer_id) = &zp.remove_peer {
        handle_remove_peer(peer_id);
        return;
    }

    if let Some(peer_id) = &zp.test_peer {
        handle_test_peer(peer_id).await;
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

// Sync command handlers
async fn handle_sync_daemon() {
    match SyncHandler::new() {
        Ok(handler) => {
            if let Err(e) = handler.start_daemon().await {
                eprintln!("❌ Failed to start sync daemon: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize sync handler: {}", e);
            process::exit(1);
        }
    }
}

async fn handle_sync_now() {
    match SyncHandler::new() {
        Ok(handler) => {
            if let Err(e) = handler.sync_now().await {
                eprintln!("❌ Failed to sync: {}", e);
                process::exit(1);
            } else {
                println!("✅ Sync completed successfully");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize sync handler: {}", e);
            process::exit(1);
        }
    }
}

fn handle_sync_config() {
    match SyncHandler::new() {
        Ok(handler) => {
            handler.show_config();
        }
        Err(e) => {
            eprintln!("❌ Failed to load sync configuration: {}", e);
            process::exit(1);
        }
    }
}

fn handle_sync_enable() {
    match SyncHandler::new() {
        Ok(mut handler) => {
            if let Err(e) = handler.enable_sync() {
                eprintln!("❌ Failed to enable sync: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize sync handler: {}", e);
            process::exit(1);
        }
    }
}

fn handle_sync_disable() {
    match SyncHandler::new() {
        Ok(mut handler) => {
            if let Err(e) = handler.disable_sync() {
                eprintln!("❌ Failed to disable sync: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize sync handler: {}", e);
            process::exit(1);
        }
    }
}

fn handle_add_peer(peer_info: &str) {
    let parts: Vec<&str> = peer_info.split(':').collect();
    if parts.len() < 2 {
        eprintln!("❌ Invalid peer format. Use: peer_id:endpoint");
        eprintln!("   Examples:");
        eprintln!("     laptop:http://192.168.1.100:8080");
        eprintln!("     server:ssh://user@server.com:22");
        process::exit(1);
    }

    let peer_id = parts[0].to_string();
    let endpoint = parts[1..].join(":");

    match SyncHandler::new() {
        Ok(mut handler) => {
            if let Err(e) = handler.add_peer(peer_id, endpoint) {
                eprintln!("❌ Failed to add peer: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize sync handler: {}", e);
            process::exit(1);
        }
    }
}

fn handle_remove_peer(peer_id: &str) {
    match SyncHandler::new() {
        Ok(mut handler) => {
            if let Err(e) = handler.remove_peer(peer_id) {
                eprintln!("❌ Failed to remove peer: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize sync handler: {}", e);
            process::exit(1);
        }
    }
}

async fn handle_test_peer(peer_id: &str) {
    match SyncHandler::new() {
        Ok(handler) => {
            if let Err(e) = handler.test_peer_connection(peer_id).await {
                eprintln!("❌ Failed to test peer: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize sync handler: {}", e);
            process::exit(1);
        }
    }
}
