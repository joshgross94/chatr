mod api;
mod commands;
mod db;
mod events;
pub mod media;
mod models;
mod network;
mod services;
mod state;

use std::sync::Arc;
use tokio::sync::{mpsc, watch};
use directories::ProjectDirs;
use libp2p::identity::Keypair;
use tracing::info;
use tauri::{Emitter, Manager};

use crate::db::Database;
use crate::events::{AppEvent, create_event_bus};
use crate::media::{MediaCommand, VoiceState};
use crate::media::frame_server::FrameServerState;
use crate::state::{AppState, ServiceContext};

fn get_data_dir(custom_dir: Option<&str>) -> std::path::PathBuf {
    if let Some(dir) = custom_dir {
        std::path::PathBuf::from(dir)
    } else {
        ProjectDirs::from("com", "chatr", "Chatr")
            .map(|p| p.data_dir().to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap().join(".chatr"))
    }
}

fn get_or_create_keypair(db: &Database) -> Keypair {
    if let Ok(Some(bytes)) = db.load_keypair() {
        if let Ok(kp) = Keypair::ed25519_from_bytes(bytes) {
            info!("Loaded existing keypair");
            return kp;
        }
    }

    let kp = Keypair::generate_ed25519();
    let bytes = kp.clone().try_into_ed25519().unwrap().to_bytes().to_vec();
    db.save_keypair(&bytes).expect("Failed to save keypair");
    info!("Generated new keypair");
    kp
}

/// Create a ServiceContext with all shared state.
fn create_service_context(
    data_dir: Option<&str>,
) -> (
    ServiceContext,
    Keypair,
    mpsc::Receiver<network::NetworkCommand>,
    mpsc::Receiver<MediaCommand>,
    watch::Sender<VoiceState>,
) {
    let data_dir = get_data_dir(data_dir);
    info!("Data directory: {:?}", data_dir);

    let db = Arc::new(Database::new(&data_dir).expect("Failed to initialize database"));
    let keypair = get_or_create_keypair(&db);
    let peer_id = libp2p::PeerId::from(keypair.public()).to_string();
    info!("My peer ID: {}", peer_id);

    let (network_tx, network_rx) = mpsc::channel::<network::NetworkCommand>(256);
    let (event_tx, _event_rx) = create_event_bus();
    let (media_tx, media_rx) = mpsc::channel::<MediaCommand>(64);
    let (voice_state_tx, voice_state_rx) = watch::channel(VoiceState::default());

    let ctx = ServiceContext {
        db,
        peer_id,
        network_tx,
        peers: Default::default(),
        room_peers: Default::default(),
        event_tx,
        media_tx,
        voice_state_rx,
    };

    (ctx, keypair, network_rx, media_rx, voice_state_tx)
}

/// Spawn the network swarm event loop.
/// Uses tauri::async_runtime::spawn in GUI mode (Tauri manages the runtime).
fn spawn_network(
    keypair: Keypair,
    network_rx: mpsc::Receiver<network::NetworkCommand>,
    ctx: &ServiceContext,
) {
    let db = ctx.db.clone();
    let event_tx = ctx.event_tx.clone();
    let peer_id = ctx.peer_id.clone();
    let peers = ctx.peers.clone();
    let room_peers = ctx.room_peers.clone();

    tauri::async_runtime::spawn(async move {
        let swarm = network::swarm::build_swarm(&keypair).expect("Failed to build swarm");
        network::swarm::run_event_loop(swarm, network_rx, db, event_tx, peer_id, peers, room_peers).await;
    });
}

/// Spawn the media engine event loop (voice/video).
/// Uses tauri::async_runtime::spawn in GUI mode.
fn spawn_media_engine(
    media_rx: mpsc::Receiver<MediaCommand>,
    voice_state_tx: watch::Sender<VoiceState>,
    frame_server: FrameServerState,
    ctx: &ServiceContext,
) {
    let network_tx = ctx.network_tx.clone();
    let event_tx = ctx.event_tx.clone();
    let peer_id = ctx.peer_id.clone();

    tauri::async_runtime::spawn(async move {
        media::engine::run_media_engine(
            media_rx,
            network_tx,
            event_tx,
            voice_state_tx,
            frame_server,
            peer_id,
        )
        .await;
    });
}

/// Spawn a bridge that forwards AppEvents to Tauri events for the GUI frontend.
fn spawn_tauri_event_bridge(app_handle: tauri::AppHandle, ctx: &ServiceContext) {
    let mut event_rx = ctx.event_tx.subscribe();

    tauri::async_runtime::spawn(async move {
        loop {
            match event_rx.recv().await {
                Ok(event) => {
                    let result = match &event {
                        AppEvent::NewMessage(msg) => app_handle.emit("new-message", msg),
                        AppEvent::PeerConnected(peer) => app_handle.emit("peer-connected", peer),
                        AppEvent::PeerDiscovered(peer) => {
                            app_handle.emit("peer-discovered", peer)
                        }
                        AppEvent::PeerDisconnected { peer_id } => {
                            app_handle.emit("peer-disconnected", serde_json::json!({ "peer_id": peer_id }))
                        }
                        AppEvent::PeerJoinedRoom { room_id, peer } => {
                            app_handle.emit("peer-joined-room", serde_json::json!({
                                "room_id": room_id,
                                "peer": peer,
                            }))
                        }
                        AppEvent::PeerLeftRoom { room_id, peer_id } => {
                            app_handle.emit("peer-left-room", serde_json::json!({
                                "room_id": room_id,
                                "peer_id": peer_id,
                            }))
                        }
                        AppEvent::MessageEdited { message_id, channel_id, new_content, edited_at } => {
                            app_handle.emit("message-edited", serde_json::json!({
                                "message_id": message_id, "channel_id": channel_id,
                                "new_content": new_content, "edited_at": edited_at,
                            }))
                        }
                        AppEvent::MessageDeleted { message_id, channel_id } => {
                            app_handle.emit("message-deleted", serde_json::json!({
                                "message_id": message_id, "channel_id": channel_id,
                            }))
                        }
                        AppEvent::ReactionAdded { message_id, channel_id, peer_id, emoji } => {
                            app_handle.emit("reaction-added", serde_json::json!({
                                "message_id": message_id, "channel_id": channel_id,
                                "peer_id": peer_id, "emoji": emoji,
                            }))
                        }
                        AppEvent::ReactionRemoved { message_id, channel_id, peer_id, emoji } => {
                            app_handle.emit("reaction-removed", serde_json::json!({
                                "message_id": message_id, "channel_id": channel_id,
                                "peer_id": peer_id, "emoji": emoji,
                            }))
                        }
                        AppEvent::TypingStarted { channel_id, peer_id, display_name } => {
                            app_handle.emit("typing-started", serde_json::json!({
                                "channel_id": channel_id, "peer_id": peer_id,
                                "display_name": display_name,
                            }))
                        }
                        AppEvent::TypingStopped { channel_id, peer_id } => {
                            app_handle.emit("typing-stopped", serde_json::json!({
                                "channel_id": channel_id, "peer_id": peer_id,
                            }))
                        }
                        AppEvent::ReadReceiptUpdated { channel_id, peer_id, last_read_message_id } => {
                            app_handle.emit("read-receipt-updated", serde_json::json!({
                                "channel_id": channel_id, "peer_id": peer_id,
                                "last_read_message_id": last_read_message_id,
                            }))
                        }
                        AppEvent::MessagePinned(pin) => app_handle.emit("message-pinned", pin),
                        AppEvent::MessageUnpinned { channel_id, message_id } => {
                            app_handle.emit("message-unpinned", serde_json::json!({
                                "channel_id": channel_id, "message_id": message_id,
                            }))
                        }
                        AppEvent::NewDmMessage(msg) => app_handle.emit("new-dm-message", msg),
                        AppEvent::FriendRequestReceived { from_peer_id, from_display_name } => {
                            app_handle.emit("friend-request-received", serde_json::json!({
                                "from_peer_id": from_peer_id,
                                "from_display_name": from_display_name,
                            }))
                        }
                        AppEvent::FriendRequestAccepted { peer_id } => {
                            app_handle.emit("friend-request-accepted", serde_json::json!({
                                "peer_id": peer_id,
                            }))
                        }
                        AppEvent::CallOfferReceived { call_id, from_peer_id, channel_id, sdp } => {
                            app_handle.emit("call-offer", serde_json::json!({
                                "call_id": call_id, "from_peer_id": from_peer_id,
                                "channel_id": channel_id, "sdp": sdp,
                            }))
                        }
                        AppEvent::CallAnswerReceived { call_id, from_peer_id, channel_id, sdp } => {
                            app_handle.emit("call-answer", serde_json::json!({
                                "call_id": call_id, "from_peer_id": from_peer_id,
                                "channel_id": channel_id, "sdp": sdp,
                            }))
                        }
                        AppEvent::IceCandidateReceived { from_peer_id, channel_id, candidate } => {
                            app_handle.emit("ice-candidate", serde_json::json!({
                                "from_peer_id": from_peer_id, "channel_id": channel_id,
                                "candidate": candidate,
                            }))
                        }
                        AppEvent::VoiceStateChanged { peer_id, display_name, channel_id, room_id, muted, deafened, video, screen_sharing } => {
                            app_handle.emit("voice-state-changed", serde_json::json!({
                                "peer_id": peer_id, "display_name": display_name,
                                "channel_id": channel_id, "room_id": room_id,
                                "muted": muted, "deafened": deafened,
                                "video": video, "screen_sharing": screen_sharing,
                            }))
                        }
                        AppEvent::VoiceConnected { peer_id } => {
                            app_handle.emit("voice-connected", serde_json::json!({
                                "peer_id": peer_id,
                            }))
                        }
                        AppEvent::VoiceDisconnected { peer_id } => {
                            app_handle.emit("voice-disconnected", serde_json::json!({
                                "peer_id": peer_id,
                            }))
                        }
                        AppEvent::SpeakingChanged { peer_id, speaking } => {
                            app_handle.emit("speaking-changed", serde_json::json!({
                                "peer_id": peer_id, "speaking": speaking,
                            }))
                        }
                        AppEvent::ChannelCreated { room_id, channel_id, name, channel_type, created_at } => {
                            app_handle.emit("channel-created", serde_json::json!({
                                "room_id": room_id, "channel_id": channel_id,
                                "name": name, "channel_type": channel_type,
                                "created_at": created_at,
                            }))
                        }
                        AppEvent::ChannelDeleted { room_id, channel_id } => {
                            app_handle.emit("channel-deleted", serde_json::json!({
                                "room_id": room_id, "channel_id": channel_id,
                            }))
                        }
                    };
                    if let Err(e) = result {
                        tracing::warn!("Failed to emit Tauri event: {}", e);
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("Tauri event bridge lagged, skipped {} events", n);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });
}

/// Run the GUI application (Tauri + API server).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    run_with_opts(None, 9847);
}

pub fn run_with_opts(data_dir: Option<&str>, api_port: u16) {
    tracing_subscriber::fmt::init();

    // Need to clone data_dir for the move closure
    let data_dir_owned = data_dir.map(|s| s.to_string());

    tauri::Builder::default()
        .setup(move |app| {
            let app_handle = app.handle().clone();

            let (ctx, keypair, network_rx, media_rx, voice_state_tx) =
                create_service_context(data_dir_owned.as_deref());

            // Manage Tauri state
            app.manage(AppState { ctx: ctx.clone(), api_port });

            // Spawn network
            spawn_network(keypair, network_rx, &ctx);

            // Create frame server state (shared between media engine and API)
            let frame_server = FrameServerState::new();

            // Spawn media engine
            spawn_media_engine(media_rx, voice_state_tx, frame_server.clone(), &ctx);

            // Spawn Tauri event bridge
            spawn_tauri_event_bridge(app_handle, &ctx);

            // Spawn API server (with frame server routes)
            let api_ctx = ctx.clone();
            let api_frame_server = frame_server.clone();
            tauri::async_runtime::spawn(async move {
                api::server::start_api_server(api_ctx, api_port, api_frame_server).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::identity::get_api_port,
            commands::identity::get_my_peer_id,
            commands::identity::get_identity,
            commands::identity::get_display_name,
            commands::identity::set_display_name,
            commands::rooms::create_room,
            commands::rooms::join_room,
            commands::rooms::list_rooms,
            commands::rooms::get_channels,
            commands::messaging::send_message,
            commands::messaging::get_messages,
            commands::messaging::get_room_peers,
            commands::voice::join_voice_channel,
            commands::voice::leave_voice_channel,
            commands::voice::set_muted,
            commands::voice::set_deafened,
            commands::voice::list_audio_devices,
            commands::voice::enable_camera,
            commands::voice::disable_camera,
            commands::voice::list_cameras,
            commands::voice::start_screen_share,
            commands::voice::stop_screen_share,
            commands::voice::send_call_offer,
            commands::voice::send_call_answer,
            commands::voice::send_ice_candidate,
            commands::voice::update_voice_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Run in headless mode (no GUI, API server only).
/// Uses tokio::spawn directly since headless mode runs on its own tokio runtime.
pub async fn run_headless(data_dir: Option<&str>, api_port: u16) {
    tracing_subscriber::fmt::init();

    let (ctx, keypair, network_rx, media_rx, voice_state_tx) = create_service_context(data_dir);

    // Spawn network (tokio::spawn since we have our own runtime in headless mode)
    let db = ctx.db.clone();
    let event_tx = ctx.event_tx.clone();
    let peer_id = ctx.peer_id.clone();
    let net_peers = ctx.peers.clone();
    let net_room_peers = ctx.room_peers.clone();
    tokio::spawn(async move {
        let swarm = network::swarm::build_swarm(&keypair).expect("Failed to build swarm");
        network::swarm::run_event_loop(swarm, network_rx, db, event_tx, peer_id, net_peers, net_room_peers).await;
    });

    // Create frame server state
    let frame_server = FrameServerState::new();

    // Spawn media engine
    let media_network_tx = ctx.network_tx.clone();
    let media_event_tx = ctx.event_tx.clone();
    let media_peer_id = ctx.peer_id.clone();
    let media_frame_server = frame_server.clone();
    tokio::spawn(async move {
        media::engine::run_media_engine(
            media_rx,
            media_network_tx,
            media_event_tx,
            voice_state_tx,
            media_frame_server,
            media_peer_id,
        )
        .await;
    });

    info!("Running in headless mode");

    // Run API server (blocks until shutdown)
    api::server::start_api_server(ctx, api_port, frame_server).await;
}
