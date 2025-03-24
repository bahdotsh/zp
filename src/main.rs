use clap::Parser;
use std::env;
use std::process;
use zp::history::print_clipboard_history;
use zp::{daemon_status, start_daemon, stop_daemon, Query, Zp};

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
