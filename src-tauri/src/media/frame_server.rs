use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Shared state for the frame server.
/// Each peer_id maps to a broadcast sender of JPEG frames + latest frame cache.
#[derive(Clone)]
pub struct FrameServerState {
    /// Video streams: peer_id -> broadcast sender of JPEG data
    pub video_streams: Arc<RwLock<HashMap<String, broadcast::Sender<Vec<u8>>>>>,
    /// Screen share streams: peer_id -> broadcast sender of JPEG data
    pub screen_streams: Arc<RwLock<HashMap<String, broadcast::Sender<Vec<u8>>>>>,
    /// Latest video frame per peer (for single-frame polling)
    latest_video_frames: Arc<RwLock<HashMap<String, Arc<Vec<u8>>>>>,
    /// Latest screen frame per peer (for single-frame polling)
    latest_screen_frames: Arc<RwLock<HashMap<String, Arc<Vec<u8>>>>>,
}

impl FrameServerState {
    pub fn new() -> Self {
        Self {
            video_streams: Arc::new(RwLock::new(HashMap::new())),
            screen_streams: Arc::new(RwLock::new(HashMap::new())),
            latest_video_frames: Arc::new(RwLock::new(HashMap::new())),
            latest_screen_frames: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new video stream for a peer. Returns a sender to push frames.
    pub async fn register_video_stream(&self, peer_id: &str) -> broadcast::Sender<Vec<u8>> {
        let (tx, _) = broadcast::channel(8);
        self.video_streams
            .write()
            .await
            .insert(peer_id.to_string(), tx.clone());
        tx
    }

    /// Remove a video stream.
    pub async fn remove_video_stream(&self, peer_id: &str) {
        self.video_streams.write().await.remove(peer_id);
        self.latest_video_frames.write().await.remove(peer_id);
    }

    /// Register a new screen share stream for a peer.
    pub async fn register_screen_stream(&self, peer_id: &str) -> broadcast::Sender<Vec<u8>> {
        let (tx, _) = broadcast::channel(8);
        self.screen_streams
            .write()
            .await
            .insert(peer_id.to_string(), tx.clone());
        tx
    }

    /// Remove a screen share stream.
    pub async fn remove_screen_stream(&self, peer_id: &str) {
        self.screen_streams.write().await.remove(peer_id);
        self.latest_screen_frames.write().await.remove(peer_id);
    }

    /// Push a video frame for a peer.
    pub async fn push_video_frame(&self, peer_id: &str, jpeg_data: Vec<u8>) {
        let frame = Arc::new(jpeg_data.clone());
        self.latest_video_frames
            .write()
            .await
            .insert(peer_id.to_string(), frame);
        let streams = self.video_streams.read().await;
        if let Some(tx) = streams.get(peer_id) {
            let _ = tx.send(jpeg_data);
        }
    }

    /// Push a screen frame for a peer.
    pub async fn push_screen_frame(&self, peer_id: &str, jpeg_data: Vec<u8>) {
        let frame = Arc::new(jpeg_data.clone());
        self.latest_screen_frames
            .write()
            .await
            .insert(peer_id.to_string(), frame);
        let streams = self.screen_streams.read().await;
        if let Some(tx) = streams.get(peer_id) {
            let _ = tx.send(jpeg_data);
        }
    }
}

/// MJPEG stream handler for video.
async fn video_stream(
    Path(peer_id): Path<String>,
    State(state): State<FrameServerState>,
) -> impl IntoResponse {
    serve_mjpeg_stream(&state.video_streams, &peer_id).await
}

/// MJPEG stream handler for screen share.
async fn screen_stream(
    Path(peer_id): Path<String>,
    State(state): State<FrameServerState>,
) -> impl IntoResponse {
    serve_mjpeg_stream(&state.screen_streams, &peer_id).await
}

/// Serve an MJPEG stream from a broadcast channel.
async fn serve_mjpeg_stream(
    streams: &Arc<RwLock<HashMap<String, broadcast::Sender<Vec<u8>>>>>,
    peer_id: &str,
) -> impl IntoResponse {
    let rx = {
        let streams = streams.read().await;
        match streams.get(peer_id) {
            Some(tx) => tx.subscribe(),
            None => {
                return axum::response::Response::builder()
                    .status(404)
                    .body(axum::body::Body::from("Stream not found"))
                    .unwrap();
            }
        }
    };

    let stream = async_stream::stream! {
        let mut rx = rx;
        // MJPEG boundary
        loop {
            match rx.recv().await {
                Ok(jpeg_data) => {
                    let header = format!(
                        "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                        jpeg_data.len()
                    );
                    yield Ok::<_, std::io::Error>(bytes::Bytes::from(header));
                    yield Ok(bytes::Bytes::from(jpeg_data));
                    yield Ok(bytes::Bytes::from("\r\n"));
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    // Skip missed frames
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    };

    axum::response::Response::builder()
        .header("Content-Type", "multipart/x-mixed-replace; boundary=frame")
        .header("Cache-Control", "no-cache, no-store, must-revalidate")
        .header("Access-Control-Allow-Origin", "*")
        .body(axum::body::Body::from_stream(stream))
        .unwrap()
}

/// Single-frame handler for video (returns latest JPEG).
async fn video_frame(
    Path(peer_id): Path<String>,
    State(state): State<FrameServerState>,
) -> impl IntoResponse {
    serve_single_frame(&state.latest_video_frames, &peer_id).await
}

/// Single-frame handler for screen share (returns latest JPEG).
async fn screen_frame(
    Path(peer_id): Path<String>,
    State(state): State<FrameServerState>,
) -> impl IntoResponse {
    serve_single_frame(&state.latest_screen_frames, &peer_id).await
}

/// Serve the latest JPEG frame for a peer.
async fn serve_single_frame(
    frames: &Arc<RwLock<HashMap<String, Arc<Vec<u8>>>>>,
    peer_id: &str,
) -> axum::response::Response {
    let frames = frames.read().await;
    match frames.get(peer_id) {
        Some(jpeg_data) => axum::response::Response::builder()
            .header("Content-Type", "image/jpeg")
            .header("Cache-Control", "no-cache, no-store, must-revalidate")
            .header("Access-Control-Allow-Origin", "*")
            .body(axum::body::Body::from(jpeg_data.as_ref().clone()))
            .unwrap(),
        None => axum::response::Response::builder()
            .status(404)
            .body(axum::body::Body::from("No frame available"))
            .unwrap(),
    }
}

/// Build the frame server router (to be nested into the main API server).
pub fn frame_server_routes(state: FrameServerState) -> Router {
    Router::new()
        .route("/media/video/:peer_id", get(video_stream))
        .route("/media/screen/:peer_id", get(screen_stream))
        .route("/media/video/:peer_id/frame", get(video_frame))
        .route("/media/screen/:peer_id/frame", get(screen_frame))
        .with_state(state)
}
