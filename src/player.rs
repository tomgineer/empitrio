// player.rs
// Non‑blocking MP3 playback using rodio. The function spawns a detached thread so the TUI remains responsive.

use std::{fs::File, io::BufReader, path::{Path, PathBuf}, thread};
use rodio::{Decoder, OutputStream, Sink};

/// Play the given MP3 file in a background thread.
/// Returns `Ok(())` immediately so the caller is non‑blocking.
/// Any I/O or audio errors are logged to stderr inside the thread.
pub fn play_file<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let path_buf: PathBuf = path.as_ref().into();

    // Spawn a detached thread so UI stays responsive.
    thread::spawn(move || {
        if let Err(e) = play_inner(&path_buf) {
            eprintln!("[audio error] {e}");
        }
    });

    Ok(())
}

fn play_inner(path: &Path) -> Result<(), String> {
    // 1. Open and decode the MP3
    let file = File::open(path).map_err(|e| format!("Failed to open {:?}: {e}", path))?;
    let source = Decoder::new(BufReader::new(file)).map_err(|e| format!("Failed to decode {:?}: {e}", path))?;

    // 2. Set up the default output device and sink
    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| format!("No output device available: {e}"))?;
    let sink = Sink::try_new(&stream_handle).map_err(|e| format!("Failed to create sink: {e}"))?;

    // 3. Play ‑‑ this blocks **inside the thread** until playback ends
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}
