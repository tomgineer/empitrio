// player.rs
// Non-blocking MP3 playback with rodio — ensures only ONE track plays at a time.
// Requires: once_cell = "1" in Cargo.toml

use rodio::{Decoder, OutputStream, Sink, Source};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};
use once_cell::sync::Lazy;

use std::sync::mpsc::Sender;
use std::time::{Instant, Duration};

// Global sink handle guarded by a mutex so we can stop the previous song
static CURRENT_SINK: Lazy<Mutex<Option<Arc<Sink>>>> = Lazy::new(|| Mutex::new(None));

/// Play the given MP3 file in a background thread, stopping any track already playing.
/// Returns immediately so the caller (TUI) remains responsive.
/// Errors are logged to stderr inside the spawned thread.
pub fn play_file<P: AsRef<Path>>(path: P, progress_sender: Sender<(u64, u64)>) -> Result<(), String> {
    let path_buf: PathBuf = path.as_ref().into();

    thread::spawn(move || {
        if let Err(e) = play_inner(&path_buf, progress_sender) {
            eprintln!("[audio error] {e}");
        }
    });

    Ok(())
}

fn play_inner(path: &Path, progress_sender: Sender<(u64, u64)>) -> Result<(), String> {
    // Stop old sink if any, ensuring only one track plays at a time
    if let Some(old_sink) = CURRENT_SINK.lock().expect("Failed to lock CURRENT_SINK").take() {
        old_sink.stop();
    }

    let file = File::open(path).map_err(|e| format!("Failed to open {path:?}: {e}"))?;
    let source = Decoder::new(BufReader::new(file)).map_err(|e| format!("Decode error: {e}"))?;

    // Get total duration in seconds or 0 if unknown
    let total_duration = source.total_duration().map(|d| d.as_secs()).unwrap_or(0);

    let (_stream, handle) = OutputStream::try_default().map_err(|e| format!("No output device: {e}"))?;
    let sink = Sink::try_new(&handle).map_err(|e| format!("Sink error: {e}"))?;

    let arc_sink = Arc::new(sink);
    arc_sink.append(source);

    // Save the Arc<Sink> so we can stop playback later if needed
    *CURRENT_SINK.lock().expect("Failed to lock CURRENT_SINK") = Some(arc_sink.clone());

    // Track playback start time
    let start = Instant::now();

    // Clone Arc<Sink> and Sender for the progress-reporting thread
    let arc_sink_clone = arc_sink.clone();
    let sender_clone = progress_sender.clone();

    thread::spawn(move || {
        while !arc_sink_clone.empty() {
            let elapsed = start.elapsed().as_secs();
            let clamped_elapsed = if total_duration > 0 && elapsed > total_duration {
                total_duration
            } else {
                elapsed
            };
            let _ = sender_clone.send((clamped_elapsed, total_duration));
            thread::sleep(Duration::from_millis(500));
        }
        // Send final update when playback finishes
        let _ = sender_clone.send((total_duration, total_duration));
    });

    // Wait for playback to finish on the original Arc<Sink>
    arc_sink.sleep_until_end();

    Ok(())
}

