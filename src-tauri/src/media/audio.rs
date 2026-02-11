use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::Serialize;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::sync::mpsc;
use tracing::{info, error};

/// Audio device info returned to the frontend/API.
#[derive(Debug, Clone, Serialize)]
pub struct AudioDevice {
    pub name: String,
    pub is_input: bool,
    pub is_default: bool,
}

/// List available input and output audio devices.
pub fn list_devices() -> Vec<AudioDevice> {
    let host = cpal::default_host();
    let mut devices = Vec::new();

    let default_input_name = host
        .default_input_device()
        .and_then(|d| d.name().ok());
    let default_output_name = host
        .default_output_device()
        .and_then(|d| d.name().ok());

    if let Ok(input_devices) = host.input_devices() {
        for device in input_devices {
            if let Ok(name) = device.name() {
                let is_default = default_input_name.as_deref() == Some(&name);
                devices.push(AudioDevice {
                    name,
                    is_input: true,
                    is_default,
                });
            }
        }
    }

    if let Ok(output_devices) = host.output_devices() {
        for device in output_devices {
            if let Ok(name) = device.name() {
                let is_default = default_output_name.as_deref() == Some(&name);
                devices.push(AudioDevice {
                    name,
                    is_input: false,
                    is_default,
                });
            }
        }
    }

    devices
}

/// Send+Sync capture handle. The cpal::Stream (which is !Send) lives on a
/// dedicated thread; we communicate via the `running` flag.
pub struct CaptureHandle {
    running: Arc<AtomicBool>,
    _thread: std::thread::JoinHandle<()>,
}

// Safety: The cpal::Stream is confined to its own thread.
// We only share the AtomicBool flag across threads.
unsafe impl Send for CaptureHandle {}
unsafe impl Sync for CaptureHandle {}

impl CaptureHandle {
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

impl Drop for CaptureHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

/// Start capturing audio from the default input device.
/// Returns a receiver of f32 PCM frames (mono, 48kHz, 960-sample chunks = 20ms).
/// The CaptureHandle must be kept alive to maintain the stream.
pub fn start_capture(
    _device_name: Option<&str>,
) -> Result<(CaptureHandle, mpsc::Receiver<Vec<f32>>), String> {
    let (tx, rx) = mpsc::channel::<Vec<f32>>(64);
    let running = Arc::new(AtomicBool::new(true));
    let running_thread = running.clone();
    let running_callback = running.clone();

    // Build the stream on a dedicated thread so the !Send cpal::Stream
    // never crosses a thread boundary.
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();

    let thread = std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = match host.default_input_device() {
            Some(d) => d,
            None => {
                let _ = ready_tx.send(Err("No input device available".into()));
                return;
            }
        };

        let device_name = device.name().unwrap_or_else(|_| "unknown".into());
        info!("Using input device: {}", device_name);

        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(48000),
            buffer_size: cpal::BufferSize::Default,
        };

        let mut buffer = Vec::with_capacity(960);

        let stream = match device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if !running_callback.load(Ordering::Relaxed) {
                    return;
                }
                for &sample in data {
                    buffer.push(sample);
                    if buffer.len() == 960 {
                        let frame = buffer.clone();
                        buffer.clear();
                        let _ = tx.try_send(frame);
                    }
                }
            },
            move |err| {
                error!("Audio capture error: {}", err);
            },
            None,
        ) {
            Ok(s) => s,
            Err(e) => {
                let _ = ready_tx.send(Err(format!("Failed to build input stream: {}", e)));
                return;
            }
        };

        if let Err(e) = stream.play() {
            let _ = ready_tx.send(Err(format!("Failed to start capture: {}", e)));
            return;
        }

        info!("Audio capture started (48kHz mono, 20ms frames)");
        let _ = ready_tx.send(Ok(()));

        // Keep the stream alive until stopped
        while running_thread.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        drop(stream);
        info!("Audio capture thread exiting");
    });

    // Wait for the stream to be ready
    match ready_rx.recv() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(e),
        Err(_) => return Err("Audio capture thread panicked".into()),
    }

    Ok((CaptureHandle { running, _thread: thread }, rx))
}

/// Send+Sync playback handle. The cpal::Stream lives on a dedicated thread.
pub struct PlaybackHandle {
    running: Arc<AtomicBool>,
    _thread: std::thread::JoinHandle<()>,
}

unsafe impl Send for PlaybackHandle {}
unsafe impl Sync for PlaybackHandle {}

impl PlaybackHandle {
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

impl Drop for PlaybackHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

/// Start audio playback on the default output device.
/// Returns a sender that accepts f32 PCM frames for playback.
/// The PlaybackHandle must be kept alive to maintain the stream.
pub fn start_playback(
    _device_name: Option<&str>,
) -> Result<(PlaybackHandle, mpsc::Sender<Vec<f32>>), String> {
    let (tx, rx) = mpsc::channel::<Vec<f32>>(64);
    let running = Arc::new(AtomicBool::new(true));
    let running_thread = running.clone();
    let running_callback = running.clone();

    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();

    let thread = std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => {
                let _ = ready_tx.send(Err("No output device available".into()));
                return;
            }
        };

        let device_name = device.name().unwrap_or_else(|_| "unknown".into());
        info!("Using output device: {}", device_name);

        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(48000),
            buffer_size: cpal::BufferSize::Default,
        };

        // Ring buffer for playback
        let ring = Arc::new(std::sync::Mutex::new(
            std::collections::VecDeque::<f32>::with_capacity(48000),
        ));
        let ring_reader = ring.clone();
        let ring_writer = ring.clone();

        // Receive frames in a background thread and push to ring buffer
        let rx = std::sync::Arc::new(std::sync::Mutex::new(rx));
        let rx_clone = rx.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("playback rx runtime");
            rt.block_on(async {
                let mut rx = rx_clone.lock().unwrap();
                while let Some(frame) = rx.recv().await {
                    let mut ring = ring_writer.lock().unwrap();
                    // Limit buffer to ~100ms to avoid latency buildup
                    while ring.len() > 4800 {
                        ring.pop_front();
                    }
                    ring.extend(frame.iter());
                }
            });
        });

        let stream = match device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                if !running_callback.load(Ordering::Relaxed) {
                    data.fill(0.0);
                    return;
                }
                let mut ring = ring_reader.lock().unwrap();
                for sample in data.iter_mut() {
                    *sample = ring.pop_front().unwrap_or(0.0);
                }
            },
            move |err| {
                error!("Audio playback error: {}", err);
            },
            None,
        ) {
            Ok(s) => s,
            Err(e) => {
                let _ = ready_tx.send(Err(format!("Failed to build output stream: {}", e)));
                return;
            }
        };

        if let Err(e) = stream.play() {
            let _ = ready_tx.send(Err(format!("Failed to start playback: {}", e)));
            return;
        }

        info!("Audio playback started (48kHz mono)");
        let _ = ready_tx.send(Ok(()));

        // Keep the stream alive until stopped
        while running_thread.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        drop(stream);
        info!("Audio playback thread exiting");
    });

    match ready_rx.recv() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(e),
        Err(_) => return Err("Audio playback thread panicked".into()),
    }

    Ok((PlaybackHandle { running, _thread: thread }, tx))
}

/// Mix multiple PCM frames (same length) by simple addition with clipping.
pub fn mix_frames(frames: &[Vec<f32>]) -> Vec<f32> {
    if frames.is_empty() {
        return Vec::new();
    }
    let len = frames[0].len();
    let mut mixed = vec![0.0f32; len];
    for frame in frames {
        for (i, &sample) in frame.iter().enumerate() {
            if i < len {
                mixed[i] += sample;
            }
        }
    }
    // Clip to [-1.0, 1.0]
    for sample in mixed.iter_mut() {
        *sample = sample.clamp(-1.0, 1.0);
    }
    mixed
}
