pub mod handler;
pub mod protocol;
pub mod server;

use crate::config::SyncConfig;
use crate::history::ClipboardHistoryEntry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncMessage {
    pub message_type: SyncMessageType,
    pub peer_id: String,
    pub timestamp: i64,
    pub data: Option<SyncData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncMessageType {
    Handshake,
    ClipboardSync,
    HistoryRequest,
    HistoryResponse,
    Heartbeat,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncData {
    ClipboardEntries(Vec<ClipboardHistoryEntry>),
    Timestamp(i64),
}

#[derive(Debug)]
pub struct SyncManager {
    config: SyncConfig,
    last_sync: HashMap<String, i64>, // peer_id -> last sync timestamp
}

impl SyncManager {
    pub fn new(config: SyncConfig) -> Self {
        Self {
            config,
            last_sync: HashMap::new(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn get_peer_id(&self) -> &str {
        &self.config.peer_id
    }

    pub fn get_listen_port(&self) -> u16 {
        self.config.listen_port
    }

    pub fn get_enabled_peers(&self) -> Vec<(&String, &crate::config::PeerConfig)> {
        self.config
            .peers
            .iter()
            .filter(|(_, peer)| peer.enabled)
            .collect()
    }

    pub fn update_last_sync(&mut self, peer_id: String, timestamp: i64) {
        self.last_sync.insert(peer_id, timestamp);
    }

    pub fn get_last_sync(&self, peer_id: &str) -> Option<i64> {
        self.last_sync.get(peer_id).copied()
    }
}

pub fn create_sync_message(
    message_type: SyncMessageType,
    peer_id: String,
    data: Option<SyncData>,
) -> SyncMessage {
    SyncMessage {
        message_type,
        peer_id,
        timestamp: chrono::Utc::now().timestamp(),
        data,
    }
}
