use crate::config::SyncConfig;
use crate::history::{load_clipboard_history, ClipboardHistoryEntry};
use crate::sync::{create_sync_message, SyncData, SyncMessage, SyncMessageType};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;

type PeerSyncState = Arc<RwLock<HashMap<String, i64>>>;

pub struct SyncServer {
    config: SyncConfig,
    sync_state: PeerSyncState,
}

impl SyncServer {
    pub fn new(config: SyncConfig) -> Self {
        Self {
            config,
            sync_state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let port = self.config.listen_port;
        let peer_id = self.config.peer_id.clone();
        let sync_state = self.sync_state.clone();

        // GET /health - Health check endpoint
        let health = warp::path("health")
            .and(warp::get())
            .map(|| warp::reply::json(&serde_json::json!({"status": "ok"})));

        // GET /peer-id - Return this peer's ID
        let peer_id_route = warp::path("peer-id")
            .and(warp::get())
            .map(move || warp::reply::json(&serde_json::json!({"peer_id": peer_id})));

        // POST /sync - Handle sync requests from other peers
        let sync_route = warp::path("sync")
            .and(warp::post())
            .and(warp::body::json())
            .and(with_sync_state(sync_state.clone()))
            .and_then(handle_sync_request);

        // GET /history - Return clipboard history (optionally since timestamp)
        let history_route = warp::path("history")
            .and(warp::get())
            .and(warp::query::<HashMap<String, String>>())
            .and_then(handle_history_request);

        let routes = health
            .or(peer_id_route)
            .or(sync_route)
            .or(history_route)
            .with(
                warp::cors()
                    .allow_any_origin()
                    .allow_headers(vec!["content-type"]),
            );

        println!("🔄 Sync server starting on port {}", port);
        warp::serve(routes).run(([0, 0, 0, 0], port)).await;

        Ok(())
    }
}

fn with_sync_state(
    sync_state: PeerSyncState,
) -> impl Filter<Extract = (PeerSyncState,), Error = Infallible> + Clone {
    warp::any().map(move || sync_state.clone())
}

async fn handle_sync_request(
    message: SyncMessage,
    sync_state: PeerSyncState,
) -> Result<impl warp::Reply, warp::Rejection> {
    match message.message_type {
        SyncMessageType::Handshake => {
            println!("🤝 Handshake from peer: {}", message.peer_id);

            let response =
                create_sync_message(SyncMessageType::Handshake, get_local_peer_id(), None);

            Ok(warp::reply::json(&response))
        }

        SyncMessageType::ClipboardSync => {
            if let Some(SyncData::ClipboardEntries(entries)) = message.data {
                println!(
                    "📋 Received {} clipboard entries from {}",
                    entries.len(),
                    message.peer_id
                );

                // Merge received entries with local history
                if let Err(e) = merge_clipboard_entries(entries).await {
                    eprintln!("Failed to merge clipboard entries: {}", e);
                }

                // Update sync state
                {
                    let mut state = sync_state.write().await;
                    state.insert(message.peer_id.clone(), message.timestamp);
                }

                let response =
                    create_sync_message(SyncMessageType::ClipboardSync, get_local_peer_id(), None);

                Ok(warp::reply::json(&response))
            } else {
                Ok(warp::reply::json(
                    &serde_json::json!({"error": "Invalid sync data"}),
                ))
            }
        }

        SyncMessageType::HistoryRequest => {
            let since_timestamp = message
                .data
                .as_ref()
                .and_then(|data| {
                    if let SyncData::Timestamp(ts) = data {
                        Some(*ts)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);

            match load_clipboard_history() {
                Ok(entries) => {
                    let filtered_entries = filter_entries_since_timestamp(entries, since_timestamp);

                    let response = create_sync_message(
                        SyncMessageType::HistoryResponse,
                        get_local_peer_id(),
                        Some(SyncData::ClipboardEntries(filtered_entries)),
                    );

                    Ok(warp::reply::json(&response))
                }
                Err(e) => {
                    eprintln!("Failed to load clipboard history: {}", e);
                    Ok(warp::reply::json(
                        &serde_json::json!({"error": "Failed to load history"}),
                    ))
                }
            }
        }

        _ => Ok(warp::reply::json(
            &serde_json::json!({"error": "Unsupported message type"}),
        )),
    }
}

async fn handle_history_request(
    params: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let since_timestamp = params
        .get("since")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    match load_clipboard_history() {
        Ok(entries) => {
            let filtered_entries = filter_entries_since_timestamp(entries, since_timestamp);
            Ok(warp::reply::json(&filtered_entries))
        }
        Err(e) => {
            eprintln!("Failed to load clipboard history: {}", e);
            Ok(warp::reply::json(
                &serde_json::json!({"error": "Failed to load history"}),
            ))
        }
    }
}

fn filter_entries_since_timestamp(
    entries: Vec<ClipboardHistoryEntry>,
    since_timestamp: i64,
) -> Vec<ClipboardHistoryEntry> {
    entries
        .into_iter()
        .filter(|entry| {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp) {
                dt.timestamp() > since_timestamp
            } else {
                false // Skip entries with invalid timestamps
            }
        })
        .collect()
}

async fn merge_clipboard_entries(
    remote_entries: Vec<ClipboardHistoryEntry>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load local history
    let local_entries = load_clipboard_history().unwrap_or_else(|_| vec![]);

    // Create a set of existing content with timestamps to avoid duplicates
    let mut existing_entries: std::collections::HashMap<String, String> = local_entries
        .iter()
        .map(|entry| (entry.content.clone(), entry.timestamp.clone()))
        .collect();

    // Collect new entries that don't already exist
    let mut new_entries = Vec::new();
    for entry in remote_entries {
        if !existing_entries.contains_key(&entry.content) {
            existing_entries.insert(entry.content.clone(), entry.timestamp.clone());
            new_entries.push(entry);
        }
    }

    let num_new = new_entries.len();
    if !new_entries.is_empty() {
        // Combine all entries and sort by timestamp
        let mut all_entries = local_entries;
        all_entries.extend(new_entries);

        all_entries.sort_by(|a, b| {
            let ts_a = chrono::DateTime::parse_from_rfc3339(&a.timestamp).unwrap_or_default();
            let ts_b = chrono::DateTime::parse_from_rfc3339(&b.timestamp).unwrap_or_default();
            ts_a.cmp(&ts_b)
        });

        // Save merged history
        save_merged_history(all_entries)?;
        println!("✅ Merged clipboard history with {} new entries", num_new);
    }

    Ok(())
}

pub fn save_merged_history(
    entries: Vec<ClipboardHistoryEntry>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    let history_dir = env::var("HOME")
        .map(|home| PathBuf::from(home).join(".zp"))
        .unwrap_or_else(|_| PathBuf::from(".zp"));

    if !history_dir.exists() {
        fs::create_dir_all(&history_dir)?;
    }

    let history_file = history_dir.join("clipboard_history.json");
    let serialized_history = serde_json::to_string_pretty(&entries)?;
    fs::write(&history_file, serialized_history)?;

    Ok(())
}

fn get_local_peer_id() -> String {
    // Try to load from config, fallback to generating one
    crate::config::SyncConfig::load()
        .map(|config| config.peer_id)
        .unwrap_or_else(|_| {
            let hostname = hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
            let random_suffix: String = (0..4)
                .map(|_| {
                    let chars = b"abcdefghijklmnopqrstuvwxyz0123456789";
                    chars[fastrand::usize(..chars.len())] as char
                })
                .collect();
            format!("{}@{}-{}", username, hostname, random_suffix)
        })
}
