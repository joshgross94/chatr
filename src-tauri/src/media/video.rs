use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution,
};
use nokhwa::Camera;
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Camera device info returned to the frontend/API.
#[derive(Debug, Clone, Serialize)]
pub struct CameraDevice {
    pub index: u32,
    pub name: String,
    pub is_default: bool,
}

/// A single video frame: JPEG-encoded bytes.
#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub jpeg_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// List available cameras.
pub fn list_cameras() -> Vec<CameraDevice> {
    match nokhwa::query(nokhwa::utils::ApiBackend::Auto) {
        Ok(devices) => devices
            .into_iter()
            .enumerate()
            .map(|(i, info)| CameraDevice {
                index: info.index().as_index().unwrap_or(i as u32),
                name: info.human_name().to_string(),
                is_default: i == 0,
            })
            .collect(),
        Err(e) => {
            warn!("Failed to query cameras: {}", e);
            Vec::new()
        }
    }
}

/// Send+Sync camera handle. The nokhwa Camera lives on a dedicated thread.
pub struct CameraHandle {
    running: Arc<AtomicBool>,
    _thread: std::thread::JoinHandle<()>,
}

unsafe impl Send for CameraHandle {}
unsafe impl Sync for CameraHandle {}

impl CameraHandle {
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

impl Drop for CameraHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

/// Start capturing video from a camera.
/// Returns JPEG-encoded frames via the channel.
/// Target: 640x480 @ 15fps.
pub fn start_camera(
    device_index: Option<u32>,
) -> Result<(CameraHandle, mpsc::Receiver<VideoFrame>), String> {
    let (tx, rx) = mpsc::channel::<VideoFrame>(16);
    let running = Arc::new(AtomicBool::new(true));
    let running_thread = running.clone();

    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();

    let thread = std::thread::spawn(move || {
        let index = match device_index {
            Some(i) => CameraIndex::Index(i),
            None => CameraIndex::Index(0),
        };

        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(
            CameraFormat::new(Resolution::new(640, 480), FrameFormat::MJPEG, 15),
        ));

        let mut camera = match Camera::new(index, requested) {
            Ok(c) => c,
            Err(e) => {
                let _ = ready_tx.send(Err(format!("Failed to open camera: {}", e)));
                return;
            }
        };

        if let Err(e) = camera.open_stream() {
            let _ = ready_tx.send(Err(format!("Failed to open camera stream: {}", e)));
            return;
        }

        let info = camera.info();
        info!("Camera started: {} ({}x{})", info.human_name(), 640, 480);
        let _ = ready_tx.send(Ok(()));

        while running_thread.load(Ordering::Relaxed) {
            match camera.frame() {
                Ok(frame) => {
                    let resolution = frame.resolution();
                    let decoded = frame.decode_image::<RgbFormat>();
                    match decoded {
                        Ok(rgb_image) => {
                            // Encode as JPEG
                            let mut jpeg_buf = Vec::new();
                            let mut cursor = std::io::Cursor::new(&mut jpeg_buf);
                            if let Err(e) = rgb_image.write_to(
                                &mut cursor,
                                image::ImageFormat::Jpeg,
                            ) {
                                error!("JPEG encode failed: {}", e);
                                continue;
                            }
                            let _ = tx.try_send(VideoFrame {
                                jpeg_data: jpeg_buf,
                                width: resolution.width(),
                                height: resolution.height(),
                            });
                        }
                        Err(e) => {
                            error!("Frame decode failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    if running_thread.load(Ordering::Relaxed) {
                        error!("Camera frame error: {}", e);
                    }
                    break;
                }
            }

            // ~15 fps
            std::thread::sleep(std::time::Duration::from_millis(66));
        }

        drop(camera);
        info!("Camera capture thread exiting");
    });

    match ready_rx.recv() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(e),
        Err(_) => return Err("Camera thread panicked".into()),
    }

    Ok((
        CameraHandle {
            running,
            _thread: thread,
        },
        rx,
    ))
}
