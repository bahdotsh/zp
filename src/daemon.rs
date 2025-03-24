use crate::history::save_clipboard_history;
use arboard::Clipboard;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::{thread, time::Duration};

pub fn start_daemon() -> Result<(), Box<dyn std::error::Error>> {
    // Check if daemon is already running
    let pid_dir = env::var("HOME")
        .map(|home| PathBuf::from(home).join(".zp"))
        .unwrap_or_else(|_| PathBuf::from(".zp"));

    if !pid_dir.exists() {
        fs::create_dir_all(&pid_dir)?;
    }

    let pid_file = pid_dir.join("zp-daemon.pid");

    if pid_file.exists() {
        let pid_str = fs::read_to_string(&pid_file)?;
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                let status = std::process::Command::new("kill")
                    .arg("-0")
                    .arg(pid.to_string())
                    .status();

                if status.is_ok() && status.unwrap().success() {
                    println!("zp daemon is already running with PID {}", pid);
                    return Ok(());
                }
            }

            #[cfg(not(unix))]
            {
                println!("Cannot verify if daemon is running, assuming it's not");
            }
        }
    }

    // Fork to background on Unix systems
    #[cfg(unix)]
    {
        use daemonize::Daemonize;
        println!("Starting zp daemon in the background");

        let pid_dir = env::var("HOME")
            .map(|home| PathBuf::from(home).join(".zp"))
            .unwrap_or_else(|_| PathBuf::from(".zp"));

        let pid_file = pid_dir.join("zp-daemon.pid");

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
                return run_daemon_worker();
            }
            Err(e) => {
                eprintln!("Error starting daemon: {}", e);
                return Err(e.into());
            }
        }
    }

    // For non-Unix systems, just continue execution
    #[cfg(not(unix))]
    {
        println!("Starting zp daemon in the foreground (background not supported on this OS)");
        return run_daemon_worker();
    }
}

// The actual daemon worker process
pub fn run_daemon_worker() -> Result<(), Box<dyn std::error::Error>> {
    // Get the pid file path
    let pid_dir = env::var("HOME")
        .map(|home| PathBuf::from(home).join(".zp"))
        .unwrap_or_else(|_| PathBuf::from(".zp"));

    let pid_file = pid_dir.join("zp-daemon.pid");

    // Write current PID to file
    let pid = process::id();
    let mut file = File::create(&pid_file)?;
    write!(file, "{}", pid)?;

    // Initialize clipboard
    let mut clipboard = Clipboard::new()?;
    let mut last_content = String::new();

    // Monitor clipboard in the background
    loop {
        match clipboard.get_text() {
            Ok(current_content) => {
                if !current_content.is_empty() && current_content != last_content {
                    save_clipboard_history(current_content.clone());
                    last_content = current_content;
                }
            }
            Err(e) => {
                eprintln!("Error reading clipboard: {}", e);
            }
        }

        thread::sleep(Duration::from_millis(500)); // Check every 500ms
    }
}

pub fn stop_daemon() -> Result<(), Box<dyn std::error::Error>> {
    let pid_dir = env::var("HOME")
        .map(|home| PathBuf::from(home).join(".zp"))
        .unwrap_or_else(|_| PathBuf::from(".zp"));

    let pid_file = pid_dir.join("zp-daemon.pid");

    if !pid_file.exists() {
        println!("zp daemon is not running");
        return Ok(());
    }

    let pid_str = fs::read_to_string(&pid_file)?;
    if let Ok(pid) = pid_str.trim().parse::<u32>() {
        // Send termination signal
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let status = std::process::Command::new("kill")
                .arg(pid.to_string())
                .status();

            if status.is_ok() && status.unwrap().success() {
                println!("Stopped zp daemon with PID {}", pid);
                // Remove PID file
                fs::remove_file(&pid_file)?;
            } else {
                println!("Failed to stop zp daemon with PID {}", pid);
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
                println!("Stopped zp daemon with PID {}", pid);
                // Remove PID file
                fs::remove_file(&pid_file)?;
            } else {
                println!("Failed to stop zp daemon with PID {}", pid);
            }
        }
    } else {
        println!("Invalid PID in daemon file");
    }

    Ok(())
}

pub fn daemon_status() -> Result<(), Box<dyn std::error::Error>> {
    let pid_dir = env::var("HOME")
        .map(|home| PathBuf::from(home).join(".zp"))
        .unwrap_or_else(|_| PathBuf::from(".zp"));

    let pid_file = pid_dir.join("zp-daemon.pid");

    if !pid_file.exists() {
        println!("zp daemon is not running");
        return Ok(());
    }

    let pid_str = fs::read_to_string(&pid_file)?;
    if let Ok(pid) = pid_str.trim().parse::<u32>() {
        // Check if process is running
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let status = std::process::Command::new("kill")
                .arg("-0")
                .arg(pid.to_string())
                .status();

            if status.is_ok() && status.unwrap().success() {
                println!("zp daemon is running with PID {}", pid);
            } else {
                println!("zp daemon is not running (stale PID file)");
                // Remove stale PID file
                fs::remove_file(&pid_file)?;
            }
        }

        // For Windows or fallback
        #[cfg(not(unix))]
        {
            println!("zp daemon appears to be running with PID {}", pid);
        }
    } else {
        println!("Invalid PID in daemon file");
    }

    Ok(())
}
