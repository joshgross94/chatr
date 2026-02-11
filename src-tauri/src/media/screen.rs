use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::mpsc;
use tracing::{error, info, warn, debug};

use super::video::VideoFrame;

/// Screen capture info.
#[derive(Debug, Clone, Serialize)]
pub struct ScreenInfo {
    pub name: String,
    pub id: String,
}

/// Send+Sync screen capture handle.
pub struct ScreenCaptureHandle {
    running: Arc<AtomicBool>,
    _thread: std::thread::JoinHandle<()>,
}

unsafe impl Send for ScreenCaptureHandle {}
unsafe impl Sync for ScreenCaptureHandle {}

impl ScreenCaptureHandle {
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

impl Drop for ScreenCaptureHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

/// Start screen capture using platform-specific methods.
pub fn start_screen_capture() -> Result<(ScreenCaptureHandle, mpsc::Receiver<VideoFrame>), String> {
    let (tx, rx) = mpsc::channel::<VideoFrame>(16);
    let running = Arc::new(AtomicBool::new(true));
    let running_thread = running.clone();

    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();

    let thread = std::thread::spawn(move || {
        match find_ffmpeg() {
            Some(ffmpeg_path) => {
                match start_ffmpeg_capture(&ffmpeg_path, &running_thread, &tx, &ready_tx) {
                    Ok(()) => {}
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                    }
                }
            }
            None => {
                let _ = ready_tx.send(Err(
                    "No screen capture method available. Install ffmpeg for screen sharing."
                        .into(),
                ));
            }
        }
    });

    match ready_rx.recv() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(e),
        Err(_) => return Err("Screen capture thread panicked".into()),
    }

    Ok((
        ScreenCaptureHandle {
            running,
            _thread: thread,
        },
        rx,
    ))
}

/// Check a list of ffmpeg candidate paths and return the first that works.
fn check_ffmpeg_candidates(candidates: Vec<String>) -> Option<String> {
    for candidate in candidates {
        if std::process::Command::new(&candidate)
            .arg("-version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok()
        {
            info!("Found ffmpeg at: {}", candidate);
            return Some(candidate);
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn find_ffmpeg() -> Option<String> {
    let mut candidates = vec![
        "ffmpeg".to_string(),
        "/usr/bin/ffmpeg".to_string(),
        "/usr/local/bin/ffmpeg".to_string(),
    ];

    if let Ok(home) = std::env::var("HOME") {
        let flatpak_base = format!("{}/.local/share/flatpak/runtime/org.freedesktop.Platform", home);
        if let Ok(entries) = std::fs::read_dir(&flatpak_base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(arch_entries) = std::fs::read_dir(&path) {
                    for arch_entry in arch_entries.flatten() {
                        if let Ok(ver_entries) = std::fs::read_dir(arch_entry.path()) {
                            for ver_entry in ver_entries.flatten() {
                                if let Ok(hash_entries) = std::fs::read_dir(ver_entry.path()) {
                                    for hash_entry in hash_entries.flatten() {
                                        let ffmpeg_path = hash_entry.path().join("files/bin/ffmpeg");
                                        if ffmpeg_path.exists() {
                                            candidates.push(ffmpeg_path.to_string_lossy().to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    check_ffmpeg_candidates(candidates)
}

#[cfg(target_os = "windows")]
fn find_ffmpeg() -> Option<String> {
    let mut candidates = vec![
        "ffmpeg".to_string(),
        r"C:\ffmpeg\bin\ffmpeg.exe".to_string(),
        r"C:\Program Files\ffmpeg\bin\ffmpeg.exe".to_string(),
        r"C:\ProgramData\chocolatey\bin\ffmpeg.exe".to_string(),
    ];

    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        candidates.push(format!(r"{}\Microsoft\WinGet\Links\ffmpeg.exe", local_app_data));
    }
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        candidates.push(format!(r"{}\scoop\shims\ffmpeg.exe", user_profile));
    }

    check_ffmpeg_candidates(candidates)
}

#[cfg(target_os = "macos")]
fn find_ffmpeg() -> Option<String> {
    let candidates = vec![
        "ffmpeg".to_string(),
        "/opt/homebrew/bin/ffmpeg".to_string(),
        "/usr/local/bin/ffmpeg".to_string(),
    ];

    check_ffmpeg_candidates(candidates)
}

/// A capture target selected by the user.
enum CaptureTarget {
    /// Capture the entire screen.
    FullScreen,
    /// Capture a specific window.
    /// - Linux: `id` = X11 window ID (hex), `title` for display
    /// - Windows: `title` = window title (used by gdigrab), `id` unused
    /// - macOS: `id` = CGWindowID, `title` for display
    Window { id: String, title: String },
}

/// Show a dialog listing available windows and "Entire Screen".
/// Returns the user's selection, or None if cancelled.
#[cfg(target_os = "linux")]
fn show_capture_dialog() -> Option<CaptureTarget> {
    use std::process::{Command, Stdio};

    let windows = get_window_list();

    if windows.is_empty() {
        info!("No windows found for dialog, defaulting to full screen");
        return Some(CaptureTarget::FullScreen);
    }

    // zenity --list
    let mut args = vec![
        "--list".to_string(),
        "--title=Share your screen".to_string(),
        "--text=Select what to share:".to_string(),
        "--column=ID".to_string(),
        "--column=Window".to_string(),
        "--hide-column=1".to_string(),
        "--width=500".to_string(),
        "--height=400".to_string(),
    ];

    args.push("__fullscreen__".to_string());
    args.push("Entire Screen".to_string());

    for (wid, name) in &windows {
        args.push(wid.clone());
        args.push(name.clone());
    }

    if let Ok(output) = Command::new("zenity")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if output.status.success() {
            let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if selection.is_empty() {
                info!("User cancelled screen share dialog");
                return None;
            }
            if selection == "__fullscreen__" {
                info!("User selected: Entire Screen");
                return Some(CaptureTarget::FullScreen);
            }
            let title = windows.iter().find(|(id, _)| id == &selection).map(|(_, n)| n.clone()).unwrap_or_default();
            info!("User selected window: {} ({})", selection, title);
            return Some(CaptureTarget::Window { id: selection, title });
        } else {
            info!("User cancelled screen share dialog (zenity exit code: {})", output.status);
            return None;
        }
    }

    // Fallback: kdialog
    let mut kd_args = vec![
        "--menu".to_string(),
        "Select what to share:".to_string(),
        "__fullscreen__".to_string(),
        "Entire Screen".to_string(),
    ];
    for (wid, name) in &windows {
        kd_args.push(wid.clone());
        kd_args.push(name.clone());
    }

    if let Ok(output) = Command::new("kdialog")
        .args(&kd_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if output.status.success() {
            let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if selection == "__fullscreen__" {
                return Some(CaptureTarget::FullScreen);
            }
            let title = windows.iter().find(|(id, _)| id == &selection).map(|(_, n)| n.clone()).unwrap_or_default();
            return Some(CaptureTarget::Window { id: selection, title });
        } else {
            return None;
        }
    }

    warn!("Neither zenity nor kdialog found. Install zenity for screen share picker. Falling back to full screen.");
    Some(CaptureTarget::FullScreen)
}

/// Show a PowerShell-based dialog for screen share target selection.
#[cfg(target_os = "windows")]
fn show_capture_dialog() -> Option<CaptureTarget> {
    use std::process::{Command, Stdio};

    let windows = get_window_list();

    if windows.is_empty() {
        info!("No windows found for dialog, defaulting to full screen");
        return Some(CaptureTarget::FullScreen);
    }

    // Build a PowerShell script that uses Out-GridView for selection
    let mut items = vec!["Entire Screen".to_string()];
    for (_id, title) in &windows {
        items.push(title.clone());
    }

    let items_str = items.iter()
        .map(|s| format!("'{}'", s.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(",");

    let script = format!(
        "@({}) | Out-GridView -Title 'Share your screen' -PassThru",
        items_str
    );

    if let Ok(output) = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if output.status.success() {
            let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if selection.is_empty() {
                info!("User cancelled screen share dialog");
                return None;
            }
            if selection == "Entire Screen" {
                info!("User selected: Entire Screen");
                return Some(CaptureTarget::FullScreen);
            }
            // Find the matching window
            if let Some((id, title)) = windows.iter().find(|(_, t)| t == &selection) {
                info!("User selected window: {}", title);
                return Some(CaptureTarget::Window { id: id.clone(), title: title.clone() });
            }
            // Partial match fallback (Out-GridView may trim)
            if let Some((id, title)) = windows.iter().find(|(_, t)| t.contains(&selection) || selection.contains(t.as_str())) {
                return Some(CaptureTarget::Window { id: id.clone(), title: title.clone() });
            }
        } else {
            info!("User cancelled screen share dialog");
            return None;
        }
    }

    warn!("PowerShell dialog failed. Falling back to full screen.");
    Some(CaptureTarget::FullScreen)
}

/// Show an AppleScript dialog for screen share target selection.
#[cfg(target_os = "macos")]
fn show_capture_dialog() -> Option<CaptureTarget> {
    use std::process::{Command, Stdio};

    let windows = get_window_list();

    if windows.is_empty() {
        info!("No windows found for dialog, defaulting to full screen");
        return Some(CaptureTarget::FullScreen);
    }

    // Build AppleScript "choose from list"
    let mut items = vec!["Entire Screen".to_string()];
    for (_id, title) in &windows {
        items.push(title.clone());
    }

    let items_str = items.iter()
        .map(|s| format!("\"{}\"", s.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(", ");

    let script = format!(
        "choose from list {{{}}} with title \"Share Screen\" with prompt \"Select what to share:\"",
        items_str
    );

    if let Ok(output) = Command::new("osascript")
        .args(["-e", &script])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if output.status.success() {
            let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if selection.is_empty() || selection == "false" {
                info!("User cancelled screen share dialog");
                return None;
            }
            if selection == "Entire Screen" {
                info!("User selected: Entire Screen");
                return Some(CaptureTarget::FullScreen);
            }
            if let Some((id, title)) = windows.iter().find(|(_, t)| t == &selection) {
                info!("User selected window: {}", title);
                return Some(CaptureTarget::Window { id: id.clone(), title: title.clone() });
            }
        } else {
            info!("User cancelled screen share dialog");
            return None;
        }
    }

    warn!("AppleScript dialog failed. Falling back to full screen.");
    Some(CaptureTarget::FullScreen)
}

/// Get a list of (window_id, window_title) for all visible windows.
#[cfg(target_os = "linux")]
fn get_window_list() -> Vec<(String, String)> {
    use std::process::{Command, Stdio};

    let mut windows = Vec::new();

    // Try wmctrl -l first (most reliable)
    if let Ok(output) = Command::new("wmctrl")
        .args(["-l"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                // Format: 0x04600003  0 hostname Window Title Here
                let parts: Vec<&str> = line.splitn(4, char::is_whitespace).collect();
                if parts.len() >= 4 {
                    let wid = parts[0].trim().to_string();
                    let desktop = parts[1].trim();
                    if desktop == "-1" {
                        continue;
                    }
                    let remainder = parts[3].trim();
                    if let Some((_host, title)) = remainder.split_once(char::is_whitespace) {
                        let title = title.trim();
                        if !title.is_empty() {
                            windows.push((wid, title.to_string()));
                        }
                    }
                }
            }
            if !windows.is_empty() {
                return windows;
            }
        }
    }

    // Fallback: use xdotool to list visible windows
    if let Ok(output) = Command::new("xdotool")
        .args(["search", "--onlyvisible", "--name", ""])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let wid = line.trim();
                if wid.is_empty() {
                    continue;
                }
                if let Ok(name_out) = Command::new("xdotool")
                    .args(["getwindowname", wid])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .output()
                {
                    if name_out.status.success() {
                        let name = String::from_utf8_lossy(&name_out.stdout).trim().to_string();
                        if !name.is_empty() {
                            if let Ok(wid_num) = wid.parse::<u64>() {
                                windows.push((format!("0x{:08x}", wid_num), name));
                            }
                        }
                    }
                }
            }
        }
    }

    windows
}

/// Get a list of (window_id, window_title) using PowerShell.
#[cfg(target_os = "windows")]
fn get_window_list() -> Vec<(String, String)> {
    use std::process::{Command, Stdio};

    let mut windows = Vec::new();

    // Get processes with visible main windows
    let script = "Get-Process | Where-Object {$_.MainWindowTitle -ne ''} | ForEach-Object { \"$($_.Id)|$($_.MainWindowTitle)\" }";

    if let Ok(output) = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let line = line.trim();
                if let Some((pid, title)) = line.split_once('|') {
                    let title = title.trim();
                    if !title.is_empty() {
                        windows.push((pid.trim().to_string(), title.to_string()));
                    }
                }
            }
        }
    }

    windows
}

/// Get a list of (window_id, window_title) using AppleScript.
#[cfg(target_os = "macos")]
fn get_window_list() -> Vec<(String, String)> {
    use std::process::{Command, Stdio};

    let mut windows = Vec::new();

    // Get visible application names with their windows
    let script = r#"tell application "System Events"
    set windowList to ""
    repeat with proc in (every process whose visible is true)
        set procName to name of proc
        set procId to unix id of proc
        try
            repeat with win in (every window of proc)
                set winName to name of win
                if winName is not "" then
                    set windowList to windowList & procId & "|" & procName & " - " & winName & linefeed
                end if
            end repeat
        end try
    end repeat
    return windowList
end tell"#;

    if let Ok(output) = Command::new("osascript")
        .args(["-e", script])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let line = line.trim();
                if let Some((id, title)) = line.split_once('|') {
                    let title = title.trim();
                    if !title.is_empty() {
                        windows.push((id.trim().to_string(), title.to_string()));
                    }
                }
            }
        }
    }

    windows
}

/// Get window geometry from an X11 window ID (Linux only).
#[cfg(target_os = "linux")]
fn get_window_geometry(wid: &str) -> Option<(i32, i32, u32, u32)> {
    use std::process::{Command, Stdio};

    // Try xdotool getwindowgeometry
    if let Ok(output) = Command::new("xdotool")
        .args(["getwindowgeometry", "--shell", wid])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut x = 0i32;
            let mut y = 0i32;
            let mut width = 0u32;
            let mut height = 0u32;

            for line in text.lines() {
                if let Some(val) = line.strip_prefix("X=") {
                    x = val.parse().unwrap_or(0);
                } else if let Some(val) = line.strip_prefix("Y=") {
                    y = val.parse().unwrap_or(0);
                } else if let Some(val) = line.strip_prefix("WIDTH=") {
                    width = val.parse().unwrap_or(0);
                } else if let Some(val) = line.strip_prefix("HEIGHT=") {
                    height = val.parse().unwrap_or(0);
                }
            }

            if width > 0 && height > 0 {
                return Some((x, y, width, height));
            }
        }
    }

    // Fallback: try xwininfo -id
    if let Ok(output) = Command::new("xwininfo")
        .args(["-id", wid])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut x = 0i32;
            let mut y = 0i32;
            let mut width = 0u32;
            let mut height = 0u32;

            for line in text.lines() {
                let line = line.trim();
                if let Some(val) = line.strip_prefix("Absolute upper-left X:") {
                    x = val.trim().parse().unwrap_or(0);
                } else if let Some(val) = line.strip_prefix("Absolute upper-left Y:") {
                    y = val.trim().parse().unwrap_or(0);
                } else if let Some(val) = line.strip_prefix("Width:") {
                    width = val.trim().parse().unwrap_or(0);
                } else if let Some(val) = line.strip_prefix("Height:") {
                    height = val.trim().parse().unwrap_or(0);
                }
            }

            if width > 0 && height > 0 {
                return Some((x, y, width, height));
            }
        }
    }

    None
}

/// Build platform-specific ffmpeg input arguments (Linux/X11).
#[cfg(target_os = "linux")]
fn build_ffmpeg_input_args(cmd: &mut std::process::Command, target: &CaptureTarget) -> Result<(), String> {
    let wayland = std::env::var("WAYLAND_DISPLAY").is_ok();

    if wayland {
        cmd.args([
            "-f", "lavfi",
            "-i", "color=c=black:s=1920x1080:r=10",
        ]);
        warn!("Wayland screen capture: placeholder only (needs xdg-desktop-portal integration)");
        return Ok(());
    }

    let x11_display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());

    match target {
        CaptureTarget::FullScreen => {
            let (sw, sh) = get_screen_resolution().unwrap_or((1920, 1080));
            let video_size = format!("{}x{}", sw, sh);
            info!("Capturing full screen: {} on {}", video_size, x11_display);
            cmd.args([
                "-f", "x11grab",
                "-framerate", "10",
                "-video_size", &video_size,
                "-i", &x11_display,
            ]);
        }
        CaptureTarget::Window { id, title } => {
            if let Some((x, y, w, h)) = get_window_geometry(id) {
                let w = w & !1;
                let h = h & !1;
                let video_size = format!("{}x{}", w, h);
                let input = format!("{}+{},{}", x11_display, x, y);
                info!("Capturing window {} '{}' ({}x{} at {},{})", id, title, w, h, x, y);
                cmd.args([
                    "-f", "x11grab",
                    "-framerate", "10",
                    "-video_size", &video_size,
                    "-i", &input,
                ]);
            } else {
                warn!("Could not get window geometry for {} '{}', falling back to full screen", id, title);
                let (sw, sh) = get_screen_resolution().unwrap_or((1920, 1080));
                let video_size = format!("{}x{}", sw, sh);
                cmd.args([
                    "-f", "x11grab",
                    "-framerate", "10",
                    "-video_size", &video_size,
                    "-i", &x11_display,
                ]);
            }
        }
    }

    Ok(())
}

/// Build platform-specific ffmpeg input arguments (Windows/gdigrab).
#[cfg(target_os = "windows")]
fn build_ffmpeg_input_args(cmd: &mut std::process::Command, target: &CaptureTarget) -> Result<(), String> {
    match target {
        CaptureTarget::FullScreen => {
            info!("Capturing full screen via gdigrab");
            cmd.args(["-f", "gdigrab", "-framerate", "10", "-i", "desktop"]);
        }
        CaptureTarget::Window { title, .. } => {
            let input = format!("title={}", title);
            info!("Capturing window '{}' via gdigrab", title);
            cmd.args(["-f", "gdigrab", "-framerate", "10", "-i", &input]);
        }
    }
    Ok(())
}

/// Build platform-specific ffmpeg input arguments (macOS/avfoundation).
#[cfg(target_os = "macos")]
fn build_ffmpeg_input_args(cmd: &mut std::process::Command, target: &CaptureTarget) -> Result<(), String> {
    // avfoundation captures the whole screen; window-level capture is not directly supported.
    match target {
        CaptureTarget::FullScreen => {
            info!("Capturing full screen via avfoundation");
        }
        CaptureTarget::Window { title, .. } => {
            info!("Window-level capture not supported on macOS avfoundation, capturing full screen (requested: '{}')", title);
        }
    }
    cmd.args([
        "-f", "avfoundation",
        "-framerate", "10",
        "-capture_cursor", "1",
        "-i", "Capture screen 0:",
    ]);
    Ok(())
}

/// Use ffmpeg to capture the screen and pipe JPEG frames.
fn start_ffmpeg_capture(
    ffmpeg_path: &str,
    running: &Arc<AtomicBool>,
    tx: &mpsc::Sender<VideoFrame>,
    ready_tx: &std::sync::mpsc::Sender<Result<(), String>>,
) -> Result<(), String> {
    use std::io::Read;
    use std::process::{Command, Stdio};

    // Show dialog to select capture target
    let target = show_capture_dialog();

    let target = match target {
        Some(t) => t,
        None => {
            return Err("Screen share cancelled by user".into());
        }
    };

    let mut cmd = Command::new(ffmpeg_path);

    // Platform-specific input arguments
    build_ffmpeg_input_args(&mut cmd, &target)?;

    // Shared output args: scale, JPEG pipe
    cmd.args([
        "-vf", "scale=1280:720:force_original_aspect_ratio=decrease,pad=1280:720:(ow-iw)/2:(oh-ih)/2",
        "-f", "image2pipe",
        "-vcodec", "mjpeg",
        "-q:v", "5",
        "-r", "10",
        "pipe:1",
    ]);

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start ffmpeg: {}", e))?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to get ffmpeg stdout".to_string())?;

    // Spawn a thread to read and log ffmpeg stderr
    if let Some(stderr) = child.stderr.take() {
        std::thread::spawn(move || {
            use std::io::BufRead;
            let reader = std::io::BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(line) => debug!("ffmpeg: {}", line),
                    Err(_) => break,
                }
            }
        });
    }

    info!("Screen capture started via ffmpeg");
    let _ = ready_tx.send(Ok(()));

    // Read JPEG frames from the pipe
    let mut buf = vec![0u8; 256 * 1024];
    let mut frame_buf = Vec::with_capacity(256 * 1024);

    while running.load(Ordering::Relaxed) {
        match stdout.read(&mut buf) {
            Ok(0) => {
                if let Ok(status) = child.wait() {
                    if !status.success() {
                        error!("ffmpeg exited with status: {}", status);
                    }
                }
                break;
            }
            Ok(n) => {
                frame_buf.extend_from_slice(&buf[..n]);

                while let Some(frame) = extract_jpeg_frame(&mut frame_buf) {
                    let _ = tx.try_send(VideoFrame {
                        jpeg_data: frame,
                        width: 1280,
                        height: 720,
                    });
                }
            }
            Err(e) => {
                if running.load(Ordering::Relaxed) {
                    error!("ffmpeg read error: {}", e);
                }
                break;
            }
        }
    }

    let _ = child.kill();
    info!("Screen capture thread exiting");
    Ok(())
}

/// Get the screen resolution via xdpyinfo (Linux).
#[cfg(target_os = "linux")]
fn get_screen_resolution() -> Option<(u32, u32)> {
    use std::process::{Command, Stdio};

    if let Ok(output) = Command::new("xdpyinfo")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let line = line.trim();
                if line.starts_with("dimensions:") {
                    if let Some(dims) = line.split_whitespace().nth(1) {
                        let parts: Vec<&str> = dims.split('x').collect();
                        if parts.len() == 2 {
                            if let (Ok(w), Ok(h)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                                return Some((w, h));
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Get the screen resolution via PowerShell (Windows).
#[cfg(target_os = "windows")]
fn get_screen_resolution() -> Option<(u32, u32)> {
    use std::process::{Command, Stdio};

    let script = "Add-Type -AssemblyName System.Windows.Forms; $s = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds; \"$($s.Width)x$($s.Height)\"";

    if let Ok(output) = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let parts: Vec<&str> = text.split('x').collect();
            if parts.len() == 2 {
                if let (Ok(w), Ok(h)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    return Some((w, h));
                }
            }
        }
    }

    None
}

/// Get the screen resolution via system_profiler (macOS).
#[cfg(target_os = "macos")]
fn get_screen_resolution() -> Option<(u32, u32)> {
    use std::process::{Command, Stdio};

    if let Ok(output) = Command::new("system_profiler")
        .args(["SPDisplaysDataType"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let line = line.trim();
                // Look for "Resolution: 2560 x 1440" or similar
                if line.starts_with("Resolution:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    // Format: "Resolution:" "2560" "x" "1440" ...
                    if parts.len() >= 4 {
                        if let (Ok(w), Ok(h)) = (parts[1].parse::<u32>(), parts[3].parse::<u32>()) {
                            return Some((w, h));
                        }
                    }
                }
            }
        }
    }

    None
}

/// Extract a complete JPEG frame from the buffer.
/// JPEG starts with 0xFF 0xD8 and ends with 0xFF 0xD9.
fn extract_jpeg_frame(buf: &mut Vec<u8>) -> Option<Vec<u8>> {
    // Find SOI marker (0xFF 0xD8)
    let start = buf.windows(2).position(|w| w == [0xFF, 0xD8])?;

    // Find EOI marker (0xFF 0xD9) after start
    let end_search = &buf[start + 2..];
    let end_offset = end_search.windows(2).position(|w| w == [0xFF, 0xD9])?;
    let end = start + 2 + end_offset + 2; // Include the EOI marker

    if end > buf.len() {
        return None;
    }

    let frame = buf[start..end].to_vec();
    buf.drain(..end);
    Some(frame)
}
