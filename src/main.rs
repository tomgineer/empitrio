use std::fs;
use std::io::BufReader;
use std::path::Path;
use rodio::{Decoder, OutputStream, Sink};

fn main() {
    // Find the first MP3 file in the current directory
    let mp3_file = fs::read_dir(".")
        .expect("Failed to read directory")
        .filter_map(Result::ok)
        .find(|entry| {
            entry.path().extension().map_or(false, |ext| ext == "mp3")
        });

    let Some(entry) = mp3_file else {
        eprintln!("No MP3 file found in current directory.");
        return;
    };

    let path = entry.path();
    println!("Playing: {}", path.display());

    // Initialize audio output stream
    let (_stream, handle) = OutputStream::try_default().expect("Failed to open audio output");
    let sink = Sink::try_new(&handle).expect("Failed to create audio sink");

    // Open and decode the MP3 file
    let file = std::fs::File::open(&path).expect("Failed to open MP3 file");
    let source = Decoder::new(BufReader::new(file)).expect("Failed to decode audio");

    sink.append(source);
    sink.sleep_until_end(); // Blocks until playback finishes
}
