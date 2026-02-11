use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch, Mutex};
use tracing::{info, warn, error, debug};
use webrtc::media::Sample;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;

use crate::events::{AppEvent, EventSender};
use crate::network::NetworkCommand;

use super::audio;
use super::codec::{OpusDecoder, OpusEncoder};
use super::frame_server::FrameServerState;
use super::peer::{PeerEvent, PeerManager};
use super::screen;
use super::video;
use super::{MediaCommand, VoiceState};

/// Run the media engine event loop.
/// This owns audio capture/playback, opus codecs, WebRTC peer connections,
/// camera, screen capture, and the frame server state.
pub async fn run_media_engine(
    mut cmd_rx: mpsc::Receiver<MediaCommand>,
    network_tx: mpsc::Sender<NetworkCommand>,
    event_tx: EventSender,
    voice_state_tx: watch::Sender<VoiceState>,
    frame_server: FrameServerState,
    my_peer_id: String,
) {
    info!("MediaEngine started for peer {}", my_peer_id);

    // Current voice session state
    let mut current_room_id: Option<String> = None;
    let mut current_channel_id: Option<String> = None;
    let mut is_muted = false;
    let mut is_deafened = false;
    let mut camera_enabled = false;
    let mut screen_sharing = false;

    // Audio I/O handles (kept alive while in voice)
    let mut capture_handle: Option<audio::CaptureHandle> = None;
    let mut capture_rx: Option<mpsc::Receiver<Vec<f32>>> = None;
    let mut playback_handle: Option<audio::PlaybackHandle> = None;
    let mut playback_tx: Option<mpsc::Sender<Vec<f32>>> = None;

    // Opus codec instances
    let mut opus_encoder: Option<OpusEncoder> = None;

    // Peer connections
    let (peer_event_tx, mut peer_event_rx) = mpsc::channel::<PeerEvent>(256);
    let mut peer_manager: Option<PeerManager> = None;

    // Per-peer opus decoders
    let remote_decoders: Arc<Mutex<HashMap<String, OpusDecoder>>> =
        Arc::new(Mutex::new(HashMap::new()));
    // Remote audio frames waiting to be mixed and played
    let remote_audio: Arc<Mutex<HashMap<String, Vec<f32>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Camera handles
    let mut camera_handle: Option<video::CameraHandle> = None;
    let mut camera_rx: Option<mpsc::Receiver<video::VideoFrame>> = None;

    // Screen capture handles
    let mut screen_handle: Option<screen::ScreenCaptureHandle> = None;
    let mut screen_rx: Option<mpsc::Receiver<video::VideoFrame>> = None;

    // Subscribe to AppEvent broadcast for incoming signaling
    let mut app_event_rx = event_tx.subscribe();

    // Voice activity detection state
    let mut speaking = false;
    let speaking_threshold: f32 = 0.01; // RMS threshold

    // Audio level tracking (for Phase D)
    let mut audio_level: f32 = 0.0;
    let audio_level_smoothing: f32 = 0.3;

    let update_state = |tx: &watch::Sender<VoiceState>,
                        room_id: &Option<String>,
                        channel_id: &Option<String>,
                        muted: bool,
                        deafened: bool,
                        cam: bool,
                        screen: bool,
                        peers: &Option<PeerManager>| {
        let state = VoiceState {
            in_voice: channel_id.is_some(),
            room_id: room_id.clone(),
            channel_id: channel_id.clone(),
            muted,
            deafened,
            camera_enabled: cam,
            screen_sharing: screen,
            connected_peers: peers
                .as_ref()
                .map(|p| p.connected_peers())
                .unwrap_or_default(),
        };
        let _ = tx.send(state);
    };

    // Helper to broadcast voice state to network
    macro_rules! broadcast_voice_state {
        () => {
            if let Some(ref room_id) = current_room_id {
                let _ = network_tx.send(NetworkCommand::SendVoiceState {
                    room_id: room_id.clone(),
                    channel_id: current_channel_id.clone(),
                    muted: is_muted,
                    deafened: is_deafened,
                    video: camera_enabled,
                    screen_sharing,
                }).await;
            }
        };
    }

    // Helper to stop camera
    macro_rules! stop_camera {
        () => {
            if camera_enabled {
                camera_handle.take();
                camera_rx.take();
                frame_server.remove_video_stream(&my_peer_id).await;
                camera_enabled = false;
                info!("Camera disabled");
            }
        };
    }

    // Helper to stop screen share
    macro_rules! stop_screen {
        () => {
            if screen_sharing {
                screen_handle.take();
                screen_rx.take();
                frame_server.remove_screen_stream(&my_peer_id).await;
                screen_sharing = false;
                info!("Screen sharing stopped");
            }
        };
    }

    loop {
        tokio::select! {
            // Process media commands from Tauri/API
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    MediaCommand::JoinVoice { room_id, channel_id } => {
                        info!("Joining voice: room={}, channel={}", room_id, channel_id);

                        // Leave current voice if any
                        if current_channel_id.is_some() {
                            if let Some(ref mut pm) = peer_manager {
                                pm.close_all().await;
                            }
                            capture_handle.take();
                            capture_rx.take();
                            playback_handle.take();
                            playback_tx.take();
                            opus_encoder.take();
                            remote_decoders.lock().await.clear();
                            remote_audio.lock().await.clear();
                            stop_camera!();
                            stop_screen!();
                        }

                        // Start audio capture
                        match audio::start_capture(None) {
                            Ok((handle, rx)) => {
                                capture_handle = Some(handle);
                                capture_rx = Some(rx);
                                info!("Audio capture started for voice channel");
                            }
                            Err(e) => {
                                warn!("Failed to start audio capture: {}. Joining voice without mic.", e);
                            }
                        }

                        // Start audio playback
                        match audio::start_playback(None) {
                            Ok((handle, tx)) => {
                                playback_handle = Some(handle);
                                playback_tx = Some(tx);
                                info!("Audio playback started for voice channel");
                            }
                            Err(e) => {
                                warn!("Failed to start audio playback: {}", e);
                            }
                        }

                        // Create opus encoder
                        match OpusEncoder::new() {
                            Ok(enc) => {
                                opus_encoder = Some(enc);
                            }
                            Err(e) => {
                                error!("Failed to create Opus encoder: {}", e);
                            }
                        }

                        // Create peer manager
                        match PeerManager::new(peer_event_tx.clone()) {
                            Ok(pm) => {
                                peer_manager = Some(pm);
                            }
                            Err(e) => {
                                error!("Failed to create PeerManager: {}", e);
                            }
                        }

                        current_room_id = Some(room_id.clone());
                        current_channel_id = Some(channel_id.clone());
                        is_muted = false;
                        is_deafened = false;
                        camera_enabled = false;
                        screen_sharing = false;

                        update_state(&voice_state_tx, &current_room_id, &current_channel_id, is_muted, is_deafened, camera_enabled, screen_sharing, &peer_manager);

                        // Broadcast voice state to room via network
                        let _ = network_tx.send(NetworkCommand::SendVoiceState {
                            room_id,
                            channel_id: Some(channel_id),
                            muted: false,
                            deafened: false,
                            video: false,
                            screen_sharing: false,
                        }).await;
                    }

                    MediaCommand::LeaveVoice => {
                        info!("Leaving voice");

                        // Broadcast that we're leaving
                        if let Some(ref room_id) = current_room_id {
                            let _ = network_tx.send(NetworkCommand::SendVoiceState {
                                room_id: room_id.clone(),
                                channel_id: None,
                                muted: false,
                                deafened: false,
                                video: false,
                                screen_sharing: false,
                            }).await;
                        }

                        // Close all peer connections
                        if let Some(ref mut pm) = peer_manager {
                            pm.close_all().await;
                        }

                        // Stop everything
                        capture_handle.take();
                        capture_rx.take();
                        playback_handle.take();
                        playback_tx.take();
                        opus_encoder.take();
                        peer_manager.take();
                        remote_decoders.lock().await.clear();
                        remote_audio.lock().await.clear();
                        stop_camera!();
                        stop_screen!();

                        current_room_id = None;
                        current_channel_id = None;
                        is_muted = false;
                        is_deafened = false;
                        speaking = false;
                        audio_level = 0.0;

                        update_state(&voice_state_tx, &current_room_id, &current_channel_id, is_muted, is_deafened, camera_enabled, screen_sharing, &peer_manager);
                    }

                    MediaCommand::SetMuted(muted) => {
                        is_muted = muted;
                        info!("Mute set to {}", muted);

                        if muted {
                            speaking = false;
                            audio_level = 0.0;
                            let _ = event_tx.send(AppEvent::SpeakingChanged {
                                peer_id: my_peer_id.clone(),
                                speaking: false,
                            });
                        }

                        update_state(&voice_state_tx, &current_room_id, &current_channel_id, is_muted, is_deafened, camera_enabled, screen_sharing, &peer_manager);
                        broadcast_voice_state!();
                    }

                    MediaCommand::SetDeafened(deafened) => {
                        is_deafened = deafened;
                        info!("Deafen set to {}", deafened);

                        update_state(&voice_state_tx, &current_room_id, &current_channel_id, is_muted, is_deafened, camera_enabled, screen_sharing, &peer_manager);
                        broadcast_voice_state!();
                    }

                    MediaCommand::EnableCamera { device_index } => {
                        if !current_channel_id.is_some() {
                            warn!("Cannot enable camera: not in voice channel");
                            continue;
                        }
                        if camera_enabled {
                            info!("Camera already enabled");
                            continue;
                        }

                        match video::start_camera(device_index) {
                            Ok((handle, rx)) => {
                                camera_handle = Some(handle);
                                camera_rx = Some(rx);
                                // Register local video stream in frame server
                                frame_server.register_video_stream(&my_peer_id).await;
                                camera_enabled = true;
                                info!("Camera enabled");

                                update_state(&voice_state_tx, &current_room_id, &current_channel_id, is_muted, is_deafened, camera_enabled, screen_sharing, &peer_manager);
                                broadcast_voice_state!();
                            }
                            Err(e) => {
                                error!("Failed to enable camera: {}", e);
                            }
                        }
                    }

                    MediaCommand::DisableCamera => {
                        stop_camera!();
                        update_state(&voice_state_tx, &current_room_id, &current_channel_id, is_muted, is_deafened, camera_enabled, screen_sharing, &peer_manager);
                        broadcast_voice_state!();
                    }

                    MediaCommand::StartScreenShare => {
                        if !current_channel_id.is_some() {
                            warn!("Cannot start screen share: not in voice channel");
                            continue;
                        }
                        if screen_sharing {
                            info!("Screen sharing already active");
                            continue;
                        }

                        match screen::start_screen_capture() {
                            Ok((handle, rx)) => {
                                screen_handle = Some(handle);
                                screen_rx = Some(rx);
                                frame_server.register_screen_stream(&my_peer_id).await;
                                screen_sharing = true;
                                info!("Screen sharing started");

                                update_state(&voice_state_tx, &current_room_id, &current_channel_id, is_muted, is_deafened, camera_enabled, screen_sharing, &peer_manager);
                                broadcast_voice_state!();
                            }
                            Err(e) => {
                                error!("Failed to start screen share: {}", e);
                            }
                        }
                    }

                    MediaCommand::StopScreenShare => {
                        stop_screen!();
                        update_state(&voice_state_tx, &current_room_id, &current_channel_id, is_muted, is_deafened, camera_enabled, screen_sharing, &peer_manager);
                        broadcast_voice_state!();
                    }
                }
            }

            // Process captured audio frames
            Some(pcm_frame) = async {
                if let Some(ref mut rx) = capture_rx {
                    rx.recv().await
                } else {
                    std::future::pending::<Option<Vec<f32>>>().await
                }
            } => {
                // Voice activity detection (RMS) — even when muted, for level meters
                let rms = (pcm_frame.iter().map(|s| s * s).sum::<f32>() / pcm_frame.len() as f32).sqrt();
                audio_level = audio_level * (1.0 - audio_level_smoothing) + rms * audio_level_smoothing;

                // Skip sending if muted
                if is_muted {
                    continue;
                }

                let now_speaking = rms > speaking_threshold;
                if now_speaking != speaking {
                    speaking = now_speaking;
                    let _ = event_tx.send(AppEvent::SpeakingChanged {
                        peer_id: my_peer_id.clone(),
                        speaking,
                    });
                }

                // Encode with Opus and write to WebRTC track
                if let Some(ref mut encoder) = opus_encoder {
                    match encoder.encode(&pcm_frame) {
                        Ok(opus_data) => {
                            if let Some(ref pm) = peer_manager {
                                let sample = Sample {
                                    data: opus_data.into(),
                                    duration: Duration::from_millis(20),
                                    ..Default::default()
                                };
                                if let Err(e) = pm.local_track().write_sample(&sample).await {
                                    debug!("Failed to write audio sample: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            debug!("Opus encode failed: {}", e);
                        }
                    }
                }
            }

            // Process camera frames
            Some(frame) = async {
                if let Some(ref mut rx) = camera_rx {
                    rx.recv().await
                } else {
                    std::future::pending::<Option<video::VideoFrame>>().await
                }
            } => {
                // Push to local frame server for preview + remote peers
                frame_server.push_video_frame(&my_peer_id, frame.jpeg_data.clone()).await;

                // Send JPEG frame to connected peers via WebRTC data channel
                if let Some(ref mut pm) = peer_manager {
                    pm.send_video_frame(&frame.jpeg_data).await;
                }
            }

            // Process screen capture frames
            Some(frame) = async {
                if let Some(ref mut rx) = screen_rx {
                    rx.recv().await
                } else {
                    std::future::pending::<Option<video::VideoFrame>>().await
                }
            } => {
                frame_server.push_screen_frame(&my_peer_id, frame.jpeg_data.clone()).await;

                // Send screen frame to connected peers via WebRTC data channel
                if let Some(ref mut pm) = peer_manager {
                    pm.send_screen_frame(&frame.jpeg_data).await;
                }
            }

            // Process peer events (WebRTC)
            Some(peer_event) = peer_event_rx.recv() => {
                match peer_event {
                    PeerEvent::ConnectionStateChanged { peer_id, state } => {
                        match state {
                            RTCPeerConnectionState::Connected => {
                                info!("WebRTC connected to {}", peer_id);
                                let _ = event_tx.send(AppEvent::VoiceConnected {
                                    peer_id: peer_id.clone(),
                                });
                                update_state(&voice_state_tx, &current_room_id, &current_channel_id, is_muted, is_deafened, camera_enabled, screen_sharing, &peer_manager);
                            }
                            RTCPeerConnectionState::Disconnected
                            | RTCPeerConnectionState::Failed
                            | RTCPeerConnectionState::Closed => {
                                info!("WebRTC disconnected from {}: {:?}", peer_id, state);
                                let _ = event_tx.send(AppEvent::VoiceDisconnected {
                                    peer_id: peer_id.clone(),
                                });
                                remote_decoders.lock().await.remove(&peer_id);
                                remote_audio.lock().await.remove(&peer_id);
                                // Remove remote peer's video/screen streams
                                frame_server.remove_video_stream(&peer_id).await;
                                frame_server.remove_screen_stream(&peer_id).await;
                                update_state(&voice_state_tx, &current_room_id, &current_channel_id, is_muted, is_deafened, camera_enabled, screen_sharing, &peer_manager);
                            }
                            _ => {}
                        }
                    }

                    PeerEvent::RemoteTrack { peer_id, track } => {
                        info!("Got remote audio track from {}", peer_id);

                        // Create decoder for this peer
                        let decoder = match OpusDecoder::new() {
                            Ok(d) => d,
                            Err(e) => {
                                error!("Failed to create decoder for {}: {}", peer_id, e);
                                continue;
                            }
                        };
                        remote_decoders.lock().await.insert(peer_id.clone(), decoder);

                        // Spawn task to read RTP packets, decode, and push to playback
                        let decoders = remote_decoders.clone();
                        let pb_tx = playback_tx.clone();
                        let pid = peer_id.clone();

                        tokio::spawn(async move {
                            let mut buf = vec![0u8; 4096];
                            loop {
                                match track.read(&mut buf).await {
                                    Ok((rtp_packet, _attributes)) => {
                                        let payload = &rtp_packet.payload;
                                        if payload.is_empty() {
                                            continue;
                                        }

                                        // Decode opus
                                        let pcm = {
                                            let mut decoders = decoders.lock().await;
                                            if let Some(decoder) = decoders.get_mut(&pid) {
                                                match decoder.decode(payload) {
                                                    Ok(pcm) => pcm,
                                                    Err(e) => {
                                                        debug!("Decode error for {}: {}", pid, e);
                                                        continue;
                                                    }
                                                }
                                            } else {
                                                break;
                                            }
                                        };

                                        // Send decoded audio to playback
                                        if let Some(ref tx) = pb_tx {
                                            let _ = tx.try_send(pcm);
                                        }
                                    }
                                    Err(e) => {
                                        debug!("Remote track read ended for {}: {}", pid, e);
                                        break;
                                    }
                                }
                            }
                            info!("Remote track reader for {} exited", pid);
                        });
                    }

                    PeerEvent::IceCandidate { peer_id, candidate } => {
                        if let Some(ref room_id) = current_room_id {
                            let _ = network_tx.send(NetworkCommand::SendIceCandidate {
                                room_id: room_id.clone(),
                                to_peer_id: peer_id,
                                channel_id: current_channel_id.clone().unwrap_or_default(),
                                candidate,
                            }).await;
                        }
                    }

                    PeerEvent::VideoFrame { peer_id, data } => {
                        // Received video frame from remote peer — push to frame server
                        debug!("Engine: received video frame ({} bytes) from {}", data.len(), peer_id);
                        frame_server.push_video_frame(&peer_id, data).await;
                    }

                    PeerEvent::ScreenFrame { peer_id, data } => {
                        // Received screen frame from remote peer — push to frame server
                        debug!("Engine: received screen frame ({} bytes) from {}", data.len(), peer_id);
                        frame_server.push_screen_frame(&peer_id, data).await;
                    }
                }
            }

            // Process incoming AppEvents (signaling from other peers)
            Ok(event) = app_event_rx.recv() => {
                // Only process if we're in a voice channel
                if current_channel_id.is_none() {
                    continue;
                }
                let channel_id = current_channel_id.as_ref().unwrap();

                match event {
                    AppEvent::VoiceStateChanged {
                        peer_id: remote_peer_id,
                        channel_id: remote_channel_id,
                        room_id: remote_room_id,
                        video,
                        screen_sharing: remote_screen,
                        ..
                    } => {
                        // A peer announced they're in the same voice channel — initiate WebRTC
                        if remote_peer_id != my_peer_id
                            && remote_channel_id.as_deref() == Some(channel_id)
                            && Some(&remote_room_id) == current_room_id.as_ref()
                        {
                            // Register remote streams if they have video/screen on
                            if video {
                                frame_server.register_video_stream(&remote_peer_id).await;
                            }
                            if remote_screen {
                                frame_server.register_screen_stream(&remote_peer_id).await;
                            }

                            // Only create offer if our peer_id is lexicographically smaller
                            // AND we don't already have a connection to this peer
                            if my_peer_id < remote_peer_id {
                                if let Some(ref mut pm) = peer_manager {
                                    if pm.has_peer(&remote_peer_id) {
                                        debug!("Already connected to {}, skipping offer", remote_peer_id);
                                        continue;
                                    }
                                    match pm.create_offer(&remote_peer_id).await {
                                        Ok(sdp) => {
                                            let _ = network_tx.send(NetworkCommand::SendCallOffer {
                                                room_id: current_room_id.clone().unwrap_or_default(),
                                                to_peer_id: remote_peer_id.clone(),
                                                call_id: uuid::Uuid::new_v4().to_string(),
                                                channel_id: channel_id.clone(),
                                                sdp,
                                            }).await;
                                        }
                                        Err(e) => {
                                            error!("Failed to create offer for {}: {}", remote_peer_id, e);
                                        }
                                    }
                                }
                            }
                        }

                        // Peer left voice — close their connection
                        if remote_peer_id != my_peer_id && remote_channel_id.is_none() {
                            if let Some(ref mut pm) = peer_manager {
                                pm.close_peer(&remote_peer_id).await;
                            }
                            remote_decoders.lock().await.remove(&remote_peer_id);
                            remote_audio.lock().await.remove(&remote_peer_id);
                            frame_server.remove_video_stream(&remote_peer_id).await;
                            frame_server.remove_screen_stream(&remote_peer_id).await;
                        }
                    }

                    AppEvent::CallOfferReceived { from_peer_id, channel_id: offer_channel_id, sdp, .. } => {
                        if &offer_channel_id == channel_id {
                            if let Some(ref mut pm) = peer_manager {
                                match pm.handle_offer(&from_peer_id, &sdp).await {
                                    Ok(answer_sdp) => {
                                        let _ = network_tx.send(NetworkCommand::SendCallAnswer {
                                            room_id: current_room_id.clone().unwrap_or_default(),
                                            to_peer_id: from_peer_id,
                                            call_id: uuid::Uuid::new_v4().to_string(),
                                            channel_id: channel_id.clone(),
                                            sdp: answer_sdp,
                                        }).await;
                                    }
                                    Err(e) => {
                                        error!("Failed to handle offer from {}: {}", from_peer_id, e);
                                    }
                                }
                            }
                        }
                    }

                    AppEvent::CallAnswerReceived { from_peer_id, channel_id: answer_channel_id, sdp, .. } => {
                        if &answer_channel_id == channel_id {
                            if let Some(ref mut pm) = peer_manager {
                                if let Err(e) = pm.handle_answer(&from_peer_id, &sdp).await {
                                    error!("Failed to handle answer from {}: {}", from_peer_id, e);
                                }
                            }
                        }
                    }

                    AppEvent::IceCandidateReceived { from_peer_id, channel_id: ice_channel_id, candidate } => {
                        if &ice_channel_id == channel_id {
                            if let Some(ref pm) = peer_manager {
                                if let Err(e) = pm.handle_ice_candidate(&from_peer_id, &candidate).await {
                                    debug!("Failed to handle ICE candidate from {}: {}", from_peer_id, e);
                                }
                            }
                        }
                    }

                    _ => {}
                }
            }
        }
    }
}
