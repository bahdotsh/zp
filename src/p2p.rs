use crate::history::{merge_clipboard_entry, merge_clipboard_history, ClipboardHistoryEntry};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, thread};
use tokio::sync::mpsc;

// Default port for zp p2p sync
const ZP_P2P_PORT: u16 = 7643;

// Message types for P2P communication
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ZpMessage {
    History(Vec<ClipboardHistoryEntry>), // Send complete history
    NewEntry(ClipboardHistoryEntry),     // Send a single new entry
    RequestHistory,                      // Request full history from peers
}

pub struct P2PNode {
    sender: mpsc::Sender<ZpMessage>,
}

// Storage for known peers
lazy_static::lazy_static! {
    static ref KNOWN_PEERS: Arc<Mutex<Vec<String>>> = {
        let p2p_dir = get_p2p_dir();
        if !p2p_dir.exists() {
            fs::create_dir_all(&p2p_dir).expect("Failed to create P2P directory");
        }

        let peers_file = p2p_dir.join("known_peers.txt");
        let peers = if peers_file.exists() {
            fs::read_to_string(&peers_file)
                .map(|content| {
                    content
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .map(String::from)
                        .collect()
                })
                .unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        };

        Arc::new(Mutex::new(peers))
    };
}

// Get the p2p directory path
fn get_p2p_dir() -> PathBuf {
    env::var("HOME")
        .map(|home| PathBuf::from(home).join(".zp").join("p2p"))
        .unwrap_or_else(|_| PathBuf::from(".zp/p2p"))
}

// Save a peer to the known peers list
fn save_peer(addr: &str) {
    let mut peers = KNOWN_PEERS.lock().unwrap();
    if !peers.contains(&addr.to_string()) {
        peers.push(addr.to_string());

        // Save to file
        let peers_file = get_p2p_dir().join("known_peers.txt");
        let content = peers.join("\n");
        let _ = fs::write(peers_file, content);
    }
}

pub fn list_peers() -> Result<(), Box<dyn Error + Send + Sync>> {
    let peers = KNOWN_PEERS.lock().unwrap();

    if peers.is_empty() {
        println!("No known peers.");
        return Ok(());
    }

    println!("Known peers:");
    for (index, peer) in peers.iter().enumerate() {
        // Try to connect to check if peer is online
        let status = match TcpStream::connect_timeout(&peer.parse()?, Duration::from_secs(1)) {
            Ok(_) => "online",
            Err(_) => "offline",
        };

        println!("  {}. {} [{}]", index + 1, peer, status);
    }

    Ok(())
}

impl P2PNode {
    pub fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let (sender, mut receiver) = mpsc::channel(32);

        // Start server in a separate thread
        thread::spawn(move || {
            if let Err(e) = start_server() {
                eprintln!("P2P server error: {}", e);
            }
        });

        // Start client handler in a separate thread
        thread::spawn(move || {
            while let Some(message) = futures::executor::block_on(receiver.recv()) {
                if let Err(e) = handle_outgoing_message(&message) {
                    eprintln!("Error sending P2P message: {}", e);
                }
            }
        });

        Ok(P2PNode { sender })
    }

    pub async fn send_new_entry(
        &self,
        entry: ClipboardHistoryEntry,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.sender.send(ZpMessage::NewEntry(entry)).await?;
        Ok(())
    }

    pub async fn request_history(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.sender.send(ZpMessage::RequestHistory).await?;
        Ok(())
    }
}

// Handle outgoing messages to all known peers
fn handle_outgoing_message(message: &ZpMessage) -> Result<(), Box<dyn Error + Send + Sync>> {
    let peers = KNOWN_PEERS.lock().unwrap().clone();

    for peer in peers {
        match TcpStream::connect_timeout(&peer.parse()?, Duration::from_secs(2)) {
            Ok(mut stream) => {
                let serialized = serde_json::to_string(&message)?;
                stream.write_all(serialized.as_bytes())?;
                stream.write_all(b"\n")?; // End of message marker

                // If requesting history, wait for response
                if let ZpMessage::RequestHistory = message {
                    let mut response = String::new();
                    let mut buffer = [0; 1024];

                    // Set read timeout
                    stream.set_read_timeout(Some(Duration::from_secs(5)))?;

                    while let Ok(bytes_read) = stream.read(&mut buffer) {
                        if bytes_read == 0 {
                            break;
                        }
                        response.push_str(&String::from_utf8_lossy(&buffer[..bytes_read]));
                        if response.ends_with('\n') {
                            break;
                        }
                    }

                    if !response.is_empty() {
                        if let Ok(ZpMessage::History(entries)) = serde_json::from_str(&response) {
                            merge_clipboard_history(entries);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to peer {}: {}", peer, e);
            }
        }
    }

    Ok(())
}

// Start the P2P server
fn start_server() -> Result<(), Box<dyn Error + Send + Sync>> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), ZP_P2P_PORT);
    let listener = TcpListener::bind(addr)?;
    println!("P2P server listening on {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                // Save peer address
                if let Ok(peer_addr) = stream.peer_addr() {
                    save_peer(&peer_addr.to_string());
                }

                let mut buffer = [0; 1024];
                let mut message = String::new();

                while let Ok(bytes_read) = stream.read(&mut buffer) {
                    if bytes_read == 0 {
                        break;
                    }

                    message.push_str(&String::from_utf8_lossy(&buffer[..bytes_read]));

                    if message.ends_with('\n') {
                        message = message.trim_end().to_string();
                        break;
                    }
                }

                if !message.is_empty() {
                    match serde_json::from_str::<ZpMessage>(&message) {
                        Ok(zp_msg) => match zp_msg {
                            ZpMessage::NewEntry(entry) => {
                                // Received a new clipboard entry from a peer
                                println!("Received new clipboard entry from peer");
                                merge_clipboard_entry(entry);
                            }
                            ZpMessage::RequestHistory => {
                                // Someone requested our history
                                println!("Peer requested our clipboard history");
                                // Load our history and send it back
                                if let Ok(history) = crate::history::load_clipboard_history() {
                                    let response =
                                        serde_json::to_string(&ZpMessage::History(history))?;
                                    stream.write_all(response.as_bytes())?;
                                    stream.write_all(b"\n")?;
                                }
                            }
                            ZpMessage::History(entries) => {
                                // Received full history from a peer
                                println!(
                                    "Received clipboard history from peer ({} entries)",
                                    entries.len()
                                );
                                merge_clipboard_history(entries);
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to parse message: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

// Connect to a specific peer
pub fn connect_to_peer(peer_addr: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Add port if not specified
    let addr = if peer_addr.contains(':') {
        peer_addr.to_string()
    } else {
        format!("{}:{}", peer_addr, ZP_P2P_PORT)
    };

    // Test connection
    match TcpStream::connect_timeout(&addr.parse()?, Duration::from_secs(5)) {
        Ok(_) => {
            println!("Successfully connected to peer {}", addr);
            save_peer(&addr);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to connect to peer {}: {}", addr, e);
            Err(e.into())
        }
    }
}

// Implement Clone for P2PNode
impl Clone for P2PNode {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}
