pub mod audio;
pub mod codec;
pub mod engine;
pub mod frame_server;
pub mod peer;
pub mod screen;
pub mod video;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Commands sent to the MediaEngine from Tauri commands / API routes.
#[derive(Debug)]
pub enum MediaCommand {
    JoinVoice {
        room_id: String,
        channel_id: String,
    },
    LeaveVoice,
    SetMuted(bool),
    SetDeafened(bool),
    EnableCamera {
        device_index: Option<u32>,
    },
    DisableCamera,
    StartScreenShare,
    StopScreenShare,
}

/// Current voice state snapshot returned by GET /voice/state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceState {
    pub in_voice: bool,
    pub room_id: Option<String>,
    pub channel_id: Option<String>,
    pub muted: bool,
    pub deafened: bool,
    pub connected_peers: Vec<String>,
    pub camera_enabled: bool,
    pub screen_sharing: bool,
}

impl Default for VoiceState {
    fn default() -> Self {
        Self {
            in_voice: false,
            room_id: None,
            channel_id: None,
            muted: false,
            deafened: false,
            connected_peers: Vec::new(),
            camera_enabled: false,
            screen_sharing: false,
        }
    }
}

/// Handle for sending commands to the media engine.
#[derive(Clone)]
pub struct MediaHandle {
    pub command_tx: mpsc::Sender<MediaCommand>,
}
