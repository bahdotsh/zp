use clap::Parser;
use is_terminal::IsTerminal;
use std::io::{self, Read};

#[derive(Parser, Debug)]
#[command(
    author = "Gokul <@bahdotsh>",
    version = env!("CARGO_PKG_VERSION"),
    about = "Tool to copy contents from a file",
    long_about = None
)]
#[command(propagate_version = true)]
pub struct Zp {
    /// Source file to read from or pattern to search for
    #[arg(default_value = "")]
    pub pattern: String,

    /// Display file names
    #[arg(short = 'f', long, default_value_t = false)]
    pub filenames: bool,

    /// Start search at lineno
    #[arg(short = 'n', long, default_value_t = 0)]
    pub lineno: usize,

    // Logs option
    #[arg(short, long, default_value_t = false)]
    pub logs: bool,

    /// Start daemon
    #[arg(long, default_value_t = false)]
    pub daemon: bool,

    /// Stop daemon
    #[arg(long, default_value_t = false)]
    pub stop_daemon: bool,

    /// Check daemon status
    #[arg(long, default_value_t = false)]
    pub daemon_status: bool,
    
    /// Enable clipboard history syncing
    #[arg(long, default_value_t = false)]
    pub sync_enable: bool,
    
    /// Disable clipboard history syncing
    #[arg(long, default_value_t = false)]
    pub sync_disable: bool,
    
    /// Show sync status
    #[arg(long, default_value_t = false)]
    pub sync_status: bool,
    
    /// Set sync directory
    #[arg(long)]
    pub sync_dir: Option<String>,
    
    /// Set device name for syncing
    #[arg(long)]
    pub sync_device_name: Option<String>,
}

pub struct Query {
    pub source: String,
    pub start: usize,
    pub end: usize,
}

impl Query {
    pub fn build(zp: &Zp) -> Result<Query, &'static str> {
        let mut source = String::new();

        if io::stdout().is_terminal() && io::stderr().is_terminal() && !io::stdin().is_terminal() {
            let mut buffer = io::stdin();
            while let Ok(n) = buffer.read_to_string(&mut source) {
                if n == 0 {
                    break;
                }
            }
        } else {
            source = if zp.pattern.is_empty() {
                return Err("No source to copy from")
            } else {
                zp.pattern.clone()
            };
        }

        let start = zp.lineno;
        let end = zp.lineno;

        Ok(Query { source, start, end })
    }
}
