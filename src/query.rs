use clap::Parser;
use is_terminal::IsTerminal;
use std::io::{self, Read};

#[derive(Parser)]
#[command(
    author = "Gokul <@bahdotsh>",
    version = env!("CARGO_PKG_VERSION"),
    about = "Tool to copy contents from a file",
)]
pub struct Zp {
    pub source: Option<String>,
    #[clap(short, long)]
    pub start: Option<usize>,
    #[clap(short, long)]
    pub end: Option<usize>,
    #[clap(short, long)]
    pub logs: bool,

    #[clap(long, short, help = "Start the clipboard monitoring daemon")]
    pub daemon: bool,
    #[clap(
        long = "stop-daemon",
        short = 'k',
        alias = "sd",
        help = "Stop the clipboard monitoring daemon"
    )]
    pub stop_daemon: bool,

    // Use short alias 't' and provide a long alias for clarity
    #[clap(
        long = "daemon-status",
        short = 't',
        alias = "ds",
        help = "Check if the daemon is running"
    )]
    pub daemon_status: bool,

    // Sync-related commands
    #[clap(long = "sync-daemon", help = "Start the sync daemon")]
    pub sync_daemon: bool,

    #[clap(long = "sync-now", help = "Sync clipboard history with peers once")]
    pub sync_now: bool,

    #[clap(long = "sync-config", help = "Show sync configuration")]
    pub sync_config: bool,

    #[clap(long = "sync-enable", help = "Enable sync")]
    pub sync_enable: bool,

    #[clap(long = "sync-disable", help = "Disable sync")]
    pub sync_disable: bool,

    #[clap(long = "add-peer", help = "Add a peer (format: peer_id:endpoint)")]
    pub add_peer: Option<String>,

    #[clap(long = "remove-peer", help = "Remove a peer by ID")]
    pub remove_peer: Option<String>,

    #[clap(long = "test-peer", help = "Test connection to a peer by ID")]
    pub test_peer: Option<String>,
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
            source = match &zp.source {
                Some(arg) => arg.to_owned(),
                None => return Err("No source to copy from"),
            };
        }

        let start = zp.start.unwrap_or(0);
        let end = zp.end.unwrap_or(0);

        Ok(Query { source, start, end })
    }
}
