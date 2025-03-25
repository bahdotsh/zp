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

    #[clap(
        long = "daemon-status",
        short = 't',
        alias = "ds",
        help = "Check if the daemon is running"
    )]
    pub daemon_status: bool,

    // P2P options
    #[clap(
        long = "p2p-sync",
        help = "Enable P2P clipboard history synchronization"
    )]
    pub p2p_sync: bool,

    #[clap(
        long = "p2p-connect",
        help = "Connect to a specific P2P node (format: /ip4/1.2.3.4/tcp/1234/p2p/PeerID)"
    )]
    pub p2p_connect: Option<String>,
    #[clap(long = "list-peers", help = "List all known P2P peers")]
    pub list_peers: bool,
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
