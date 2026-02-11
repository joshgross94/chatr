use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{info, warn, debug};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine as WrtcMediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::RTPCodecType;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;
use webrtc::track::track_remote::TrackRemote;

/// Max data channel message size (under 16KB SCTP limit).
const MAX_DC_MSG_SIZE: usize = 15000;
/// Chunk header: 'C' + original_type(1) + frame_id(4) + total_chunks(2) + chunk_index(2) = 10 bytes.
const CHUNK_HEADER_SIZE: usize = 10;
/// Max payload data per chunk.
const MAX_CHUNK_DATA: usize = MAX_DC_MSG_SIZE - CHUNK_HEADER_SIZE;

/// Events emitted by peer connections back to the engine.
#[derive(Debug)]
pub enum PeerEvent {
    /// WebRTC connection state changed.
    ConnectionStateChanged {
        peer_id: String,
        state: RTCPeerConnectionState,
    },
    /// Received an audio track from a remote peer.
    RemoteTrack {
        peer_id: String,
        track: Arc<TrackRemote>,
    },
    /// ICE candidate gathered — must be sent to the remote peer.
    IceCandidate {
        peer_id: String,
        candidate: String,
    },
    /// Received a video frame from a remote peer via data channel.
    VideoFrame {
        peer_id: String,
        data: Vec<u8>,
    },
    /// Received a screen share frame from a remote peer via data channel.
    ScreenFrame {
        peer_id: String,
        data: Vec<u8>,
    },
}

/// State for reassembling chunked frames from a remote peer.
#[derive(Default)]
struct ChunkAssembler {
    /// (peer_id, frame_type, frame_id) -> (total_chunks, received_chunks)
    pending: HashMap<(String, u8, u32), (u16, HashMap<u16, Vec<u8>>)>,
}

impl ChunkAssembler {
    fn add_chunk(&mut self, peer_id: &str, frame_type: u8, frame_id: u32, total_chunks: u16, chunk_index: u16, data: Vec<u8>) -> Option<(String, u8, Vec<u8>)> {
        let key = (peer_id.to_string(), frame_type, frame_id);
        let entry = self.pending.entry(key.clone()).or_insert_with(|| (total_chunks, HashMap::new()));
        entry.1.insert(chunk_index, data);

        if entry.1.len() == total_chunks as usize {
            // All chunks received — reassemble in order
            let (_, chunks) = self.pending.remove(&key).unwrap();
            let mut full_data = Vec::new();
            for i in 0..total_chunks {
                if let Some(chunk) = chunks.get(&i) {
                    full_data.extend_from_slice(chunk);
                }
            }
            Some((peer_id.to_string(), frame_type, full_data))
        } else {
            None
        }
    }

    /// Discard stale partial frames (keep only last 4 frame_ids per peer+type).
    fn cleanup(&mut self, peer_id: &str, frame_type: u8, current_frame_id: u32) {
        self.pending.retain(|k, _| {
            k.0 != peer_id || k.1 != frame_type || current_frame_id.wrapping_sub(k.2) < 4
        });
    }
}

/// Manages all WebRTC peer connections for voice.
pub struct PeerManager {
    connections: HashMap<String, Arc<RTCPeerConnection>>,
    /// Data channels for sending video/screen frames (peer_id -> channel).
    /// Shared with on_data_channel callbacks so both offerer and answerer can send.
    data_channels: Arc<Mutex<HashMap<String, Arc<RTCDataChannel>>>>,
    local_track: Arc<TrackLocalStaticSample>,
    event_tx: mpsc::Sender<PeerEvent>,
    /// Incrementing frame ID counter for chunked sends.
    frame_counter: u32,
}

impl PeerManager {
    /// Create a new PeerManager with a local audio track.
    pub fn new(event_tx: mpsc::Sender<PeerEvent>) -> Result<Self, String> {
        let local_track = Arc::new(TrackLocalStaticSample::new(
            webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability {
                mime_type: "audio/opus".to_string(),
                clock_rate: 48000,
                channels: 1,
                sdp_fmtp_line: "minptime=10;useinbandfec=1".to_string(),
                rtcp_feedback: vec![],
            },
            "audio-track".to_string(),
            "chatr-voice".to_string(),
        ));

        Ok(Self {
            connections: HashMap::new(),
            data_channels: Arc::new(Mutex::new(HashMap::new())),
            local_track,
            event_tx,
            frame_counter: 0,
        })
    }

    /// Get a reference to the local audio track for writing samples.
    pub fn local_track(&self) -> &Arc<TrackLocalStaticSample> {
        &self.local_track
    }

    /// Send a video frame to all connected peers via data channels.
    pub async fn send_video_frame(&mut self, jpeg_data: &[u8]) {
        self.send_frame(b'V', jpeg_data).await;
    }

    /// Send a screen share frame to all connected peers via data channels.
    pub async fn send_screen_frame(&mut self, jpeg_data: &[u8]) {
        self.send_frame(b'S', jpeg_data).await;
    }

    /// Send a frame (video or screen) with automatic chunking for large frames.
    async fn send_frame(&mut self, type_byte: u8, jpeg_data: &[u8]) {
        let channels = self.data_channels.lock().await;
        if channels.is_empty() {
            return;
        }

        // Small frame: send as single message (type_byte + data)
        if 1 + jpeg_data.len() <= MAX_DC_MSG_SIZE {
            let mut msg = Vec::with_capacity(1 + jpeg_data.len());
            msg.push(type_byte);
            msg.extend_from_slice(jpeg_data);
            let data = bytes::Bytes::from(msg);
            for (pid, dc) in channels.iter() {
                if let Err(e) = dc.send(&data).await {
                    debug!("Failed to send frame to {}: {}", pid, e);
                }
            }
            return;
        }

        // Large frame: chunk it
        let frame_id = self.frame_counter;
        self.frame_counter = self.frame_counter.wrapping_add(1);
        let total_chunks = ((jpeg_data.len() + MAX_CHUNK_DATA - 1) / MAX_CHUNK_DATA) as u16;

        for chunk_idx in 0..total_chunks {
            let start = chunk_idx as usize * MAX_CHUNK_DATA;
            let end = std::cmp::min(start + MAX_CHUNK_DATA, jpeg_data.len());
            let chunk_data = &jpeg_data[start..end];

            let mut msg = Vec::with_capacity(CHUNK_HEADER_SIZE + chunk_data.len());
            msg.push(b'C'); // Chunked message marker
            msg.push(type_byte);
            msg.extend_from_slice(&frame_id.to_le_bytes());
            msg.extend_from_slice(&total_chunks.to_le_bytes());
            msg.extend_from_slice(&chunk_idx.to_le_bytes());
            msg.extend_from_slice(chunk_data);

            let data = bytes::Bytes::from(msg);
            for (pid, dc) in channels.iter() {
                if let Err(e) = dc.send(&data).await {
                    debug!("Failed to send chunk {}/{} to {}: {}", chunk_idx + 1, total_chunks, pid, e);
                }
            }
        }
    }

    /// Create a new peer connection and return its SDP offer.
    pub async fn create_offer(&mut self, peer_id: &str) -> Result<String, String> {
        let pc = self.create_peer_connection(peer_id).await?;

        // Create data channel for video/screen frames (offerer creates it)
        let dc = pc
            .create_data_channel("media-frames", None)
            .await
            .map_err(|e| format!("Failed to create data channel: {}", e))?;

        Self::setup_data_channel_shared(&self.data_channels, &self.event_tx, peer_id, dc);

        let offer = pc
            .create_offer(None)
            .await
            .map_err(|e| format!("Failed to create offer: {}", e))?;

        pc.set_local_description(offer.clone())
            .await
            .map_err(|e| format!("Failed to set local description: {}", e))?;

        let sdp = serde_json::to_string(&offer)
            .map_err(|e| format!("Failed to serialize SDP: {}", e))?;

        info!("Created WebRTC offer for peer {}", peer_id);
        Ok(sdp)
    }

    /// Handle an incoming SDP offer and return an answer.
    pub async fn handle_offer(&mut self, peer_id: &str, sdp_json: &str) -> Result<String, String> {
        let offer: RTCSessionDescription = serde_json::from_str(sdp_json)
            .map_err(|e| format!("Failed to parse offer SDP: {}", e))?;

        // Create connection if it doesn't exist
        if !self.connections.contains_key(peer_id) {
            self.create_peer_connection(peer_id).await?;
        }

        let pc = self.connections.get(peer_id)
            .ok_or_else(|| "No peer connection".to_string())?;

        pc.set_remote_description(offer)
            .await
            .map_err(|e| format!("Failed to set remote description: {}", e))?;

        let answer = pc
            .create_answer(None)
            .await
            .map_err(|e| format!("Failed to create answer: {}", e))?;

        pc.set_local_description(answer.clone())
            .await
            .map_err(|e| format!("Failed to set local description: {}", e))?;

        let sdp = serde_json::to_string(&answer)
            .map_err(|e| format!("Failed to serialize answer SDP: {}", e))?;

        info!("Created WebRTC answer for peer {}", peer_id);
        Ok(sdp)
    }

    /// Handle an incoming SDP answer.
    pub async fn handle_answer(&mut self, peer_id: &str, sdp_json: &str) -> Result<(), String> {
        let answer: RTCSessionDescription = serde_json::from_str(sdp_json)
            .map_err(|e| format!("Failed to parse answer SDP: {}", e))?;

        let pc = self.connections.get(peer_id)
            .ok_or_else(|| format!("No peer connection for {}", peer_id))?;

        pc.set_remote_description(answer)
            .await
            .map_err(|e| format!("Failed to set remote description: {}", e))?;

        info!("Applied WebRTC answer from peer {}", peer_id);
        Ok(())
    }

    /// Handle an incoming ICE candidate.
    pub async fn handle_ice_candidate(&self, peer_id: &str, candidate_json: &str) -> Result<(), String> {
        let candidate: webrtc::ice_transport::ice_candidate::RTCIceCandidateInit =
            serde_json::from_str(candidate_json)
                .map_err(|e| format!("Failed to parse ICE candidate: {}", e))?;

        let pc = self.connections.get(peer_id)
            .ok_or_else(|| format!("No peer connection for {}", peer_id))?;

        pc.add_ice_candidate(candidate)
            .await
            .map_err(|e| format!("Failed to add ICE candidate: {}", e))?;

        debug!("Added ICE candidate from peer {}", peer_id);
        Ok(())
    }

    /// Close a specific peer connection.
    pub async fn close_peer(&mut self, peer_id: &str) {
        self.data_channels.lock().await.remove(peer_id);
        if let Some(pc) = self.connections.remove(peer_id) {
            if let Err(e) = pc.close().await {
                warn!("Error closing peer connection to {}: {}", peer_id, e);
            }
            info!("Closed peer connection to {}", peer_id);
        }
    }

    /// Close all peer connections.
    pub async fn close_all(&mut self) {
        let peer_ids: Vec<String> = self.connections.keys().cloned().collect();
        for peer_id in peer_ids {
            self.close_peer(&peer_id).await;
        }
    }

    /// Get list of connected peer IDs.
    pub fn connected_peers(&self) -> Vec<String> {
        self.connections.keys().cloned().collect()
    }

    /// Check if we already have a connection to this peer.
    pub fn has_peer(&self, peer_id: &str) -> bool {
        self.connections.contains_key(peer_id)
    }

    /// Set up data channel event handlers for receiving video/screen frames,
    /// and store the channel for sending.
    fn setup_data_channel_shared(
        data_channels: &Arc<Mutex<HashMap<String, Arc<RTCDataChannel>>>>,
        event_tx: &mpsc::Sender<PeerEvent>,
        peer_id: &str,
        dc: Arc<RTCDataChannel>,
    ) {
        let channels = data_channels.clone();

        // Store for sending (both offerer and answerer)
        let pid_store = peer_id.to_string();
        let dc_store = dc.clone();
        tokio::spawn(async move {
            channels.lock().await.insert(pid_store, dc_store);
        });

        Self::setup_on_message(event_tx, peer_id, dc);
    }

    /// Set up on_message handler with chunk reassembly support.
    fn setup_on_message(
        event_tx: &mpsc::Sender<PeerEvent>,
        peer_id: &str,
        dc: Arc<RTCDataChannel>,
    ) {
        let event_tx = event_tx.clone();
        let pid = peer_id.to_string();
        let assembler = Arc::new(Mutex::new(ChunkAssembler::default()));

        dc.on_message(Box::new(move |msg| {
            let tx = event_tx.clone();
            let pid = pid.clone();
            let assembler = assembler.clone();
            Box::pin(async move {
                let data = msg.data.to_vec();
                if data.is_empty() {
                    return;
                }
                match data[0] {
                    // Single-message frame (small enough to fit in one DC message)
                    b'V' => {
                        let _ = tx.send(PeerEvent::VideoFrame {
                            peer_id: pid,
                            data: data[1..].to_vec(),
                        }).await;
                    }
                    b'S' => {
                        let _ = tx.send(PeerEvent::ScreenFrame {
                            peer_id: pid,
                            data: data[1..].to_vec(),
                        }).await;
                    }
                    // Chunked message (large frame split into multiple DC messages)
                    b'C' => {
                        if data.len() < CHUNK_HEADER_SIZE {
                            return;
                        }
                        let frame_type = data[1];
                        let frame_id = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
                        let total_chunks = u16::from_le_bytes([data[6], data[7]]);
                        let chunk_index = u16::from_le_bytes([data[8], data[9]]);
                        let chunk_data = data[CHUNK_HEADER_SIZE..].to_vec();

                        let mut asm = assembler.lock().await;
                        asm.cleanup(&pid, frame_type, frame_id);

                        if let Some((peer_id, ft, full_data)) = asm.add_chunk(&pid, frame_type, frame_id, total_chunks, chunk_index, chunk_data) {
                            match ft {
                                b'V' => {
                                    let _ = tx.send(PeerEvent::VideoFrame { peer_id, data: full_data }).await;
                                }
                                b'S' => {
                                    let _ = tx.send(PeerEvent::ScreenFrame { peer_id, data: full_data }).await;
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {
                        debug!("Unknown data channel message type: {}", data[0]);
                    }
                }
            })
        }));
    }

    /// Internal: create a new RTCPeerConnection with audio track.
    async fn create_peer_connection(&mut self, peer_id: &str) -> Result<Arc<RTCPeerConnection>, String> {
        // Close existing connection to this peer if any
        if self.connections.contains_key(peer_id) {
            self.close_peer(peer_id).await;
        }

        let mut media_engine = WrtcMediaEngine::default();
        media_engine
            .register_default_codecs()
            .map_err(|e| format!("Failed to register codecs: {}", e))?;

        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut media_engine)
            .map_err(|e| format!("Failed to register interceptors: {}", e))?;

        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .build();

        let config = RTCConfiguration {
            ice_servers: vec![
                RTCIceServer {
                    urls: vec![
                        "stun:stun.l.google.com:19302".to_string(),
                        "stun:stun1.l.google.com:19302".to_string(),
                    ],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let pc = Arc::new(
            api.new_peer_connection(config)
                .await
                .map_err(|e| format!("Failed to create peer connection: {}", e))?,
        );

        // Add local audio track
        let rtp_sender = pc
            .add_track(self.local_track.clone() as Arc<dyn TrackLocal + Send + Sync>)
            .await
            .map_err(|e| format!("Failed to add audio track: {}", e))?;

        // Read incoming RTCP packets (needed by webrtc crate for proper operation)
        tokio::spawn(async move {
            let mut buf = vec![0u8; 1500];
            while rtp_sender.read(&mut buf).await.is_ok() {}
        });

        // Set up event handlers
        let event_tx = self.event_tx.clone();
        let pid = peer_id.to_string();

        // Connection state change
        let event_tx_state = event_tx.clone();
        let pid_state = pid.clone();
        pc.on_peer_connection_state_change(Box::new(move |state: RTCPeerConnectionState| {
            let tx = event_tx_state.clone();
            let pid = pid_state.clone();
            Box::pin(async move {
                info!("WebRTC connection to {} state: {}", pid, state);
                let _ = tx.send(PeerEvent::ConnectionStateChanged {
                    peer_id: pid,
                    state,
                }).await;
            })
        }));

        // On track (remote audio)
        let event_tx_track = event_tx.clone();
        let pid_track = pid.clone();
        pc.on_track(Box::new(move |track, _receiver, _transceiver| {
            let tx = event_tx_track.clone();
            let pid = pid_track.clone();
            Box::pin(async move {
                if track.kind() == RTPCodecType::Audio {
                    info!("Received remote audio track from {}", pid);
                    let _ = tx.send(PeerEvent::RemoteTrack {
                        peer_id: pid,
                        track,
                    }).await;
                }
            })
        }));

        // ICE candidate gathering
        let event_tx_ice = event_tx.clone();
        let pid_ice = pid.clone();
        pc.on_ice_candidate(Box::new(move |candidate| {
            let tx = event_tx_ice.clone();
            let pid = pid_ice.clone();
            Box::pin(async move {
                if let Some(candidate) = candidate {
                    let json = match candidate.to_json() {
                        Ok(init) => serde_json::to_string(&init).unwrap_or_default(),
                        Err(e) => {
                            warn!("Failed to serialize ICE candidate: {}", e);
                            return;
                        }
                    };
                    let _ = tx.send(PeerEvent::IceCandidate {
                        peer_id: pid,
                        candidate: json,
                    }).await;
                }
            })
        }));

        // On data channel (answerer receives it — also store for sending back)
        let event_tx_dc = event_tx.clone();
        let pid_dc = pid.clone();
        let dc_channels = self.data_channels.clone();
        pc.on_data_channel(Box::new(move |dc| {
            let tx = event_tx_dc.clone();
            let pid = pid_dc.clone();
            let channels = dc_channels.clone();
            Box::pin(async move {
                info!("Received data channel '{}' from {}", dc.label(), pid);
                if dc.label() == "media-frames" {
                    // Store for sending (answerer side)
                    channels.lock().await.insert(pid.clone(), dc.clone());
                    // Reuse the same chunk-aware on_message handler
                    Self::setup_on_message(&tx, &pid, dc);
                }
            })
        }));

        self.connections.insert(peer_id.to_string(), pc.clone());
        info!("Created WebRTC peer connection for {}", peer_id);

        Ok(pc)
    }
}
