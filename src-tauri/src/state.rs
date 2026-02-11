use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, watch, Mutex as TokioMutex};

use crate::db::Database;
use crate::events::EventSender;
use crate::media::{MediaCommand, VoiceState};
use crate::models::PeerInfo;
use crate::network::NetworkCommand;

/// Transport-agnostic context shared by services, API routes, and Tauri commands.
#[derive(Clone)]
pub struct ServiceContext {
    pub db: Arc<Database>,
    pub peer_id: String,
    pub network_tx: mpsc::Sender<NetworkCommand>,
    pub peers: Arc<TokioMutex<HashMap<String, PeerInfo>>>,
    /// Tracks which peers are in which rooms (room_id -> set of peer_ids)
    pub room_peers: Arc<TokioMutex<HashMap<String, HashSet<String>>>>,
    pub event_tx: EventSender,
    pub media_tx: mpsc::Sender<MediaCommand>,
    pub voice_state_rx: watch::Receiver<VoiceState>,
}

/// Tauri-managed state that wraps ServiceContext.
pub struct AppState {
    pub ctx: ServiceContext,
    pub api_port: u16,
}
