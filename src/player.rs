// player.rs
// Non‑blocking MP3 playback with rodio — ensures only ONE track plays at a time.
// Requires: once_cell = "1" in Cargo.toml

use rodio::{Decoder, OutputStream, Sink};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};

use once_cell::sync::Lazy;

// Global sink handle guarded by a mutex so we can stop the previous song
static CURRENT_SINK: Lazy<Mutex<Option<Arc<Sink>>>> = Lazy::new(|| Mutex::new(None));

/// Play the given MP3 file in a background thread, stopping any track already playing.
/// Returns immediately so the caller (TUI) remains responsive.
/// Errors are logged to stderr inside the spawned thread.
pub fn play_file<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let path_buf: PathBuf = path.as_ref().into();

    // Spawn detached thread for audio playback
    thread::spawn(move || {
        if let Err(e) = play_inner(&path_buf) {
            eprintln!("[audio error] {e}");
        }
    });

    Ok(())
}

fn play_inner(path: &Path) -> Result<(), String> {
    // 1️⃣ Stop any previously playing track
    if let Some(old_sink) = CURRENT_SINK.lock().unwrap().take() {
        old_sink.stop();
    }

    // 2️⃣ Decode the new file
    let file = File::open(path).map_err(|e| format!("Failed to open {path:?}: {e}"))?;
    let source = Decoder::new(BufReader::new(file)).map_err(|e| format!("Decode error: {e}"))?;

    // 3️⃣ Set up audio output
    let (_stream, handle) = OutputStream::try_default().map_err(|e| format!("No output device: {e}"))?;
    let sink = Sink::try_new(&handle).map_err(|e| format!("Sink error: {e}"))?;

    // 4️⃣ Wrap sink in Arc so we can share/stop it
    let arc_sink = Arc::new(sink);
    arc_sink.append(source);

    // Save for future stop
    *CURRENT_SINK.lock().unwrap() = Some(arc_sink.clone());

    // Block inside thread until this track ends
    arc_sink.sleep_until_end();
    Ok(())
}
