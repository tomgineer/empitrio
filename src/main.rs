// main.rs
// Empitrio – minimal TUI that lists MP3 files and shows the chosen filename in the status bar.
// Dependencies (from Cargo.toml): rodio = "0.20", crossterm = "0.29", ratatui = "0.29"

use std::{env, fs, io};
use std::sync::mpsc::{Receiver, Sender};

mod player;
use player::play_file;

mod theme;
mod ui;
use ui::ui_loop;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

/// Application state
pub struct App {
    files: Vec<String>,         // List of .mp3 files in the current directory
    selected: usize,            // Index of the currently highlighted/selected file in the list
    status: String,             // Message shown in the status bar (e.g., "Playing", "Paused")
    pub current_time: u64,      // Elapsed playback time of the current song, in seconds
    pub total_time: u64,        // Total duration of the current song, in seconds
    pub perc_played: f32,       // Percentage of the current song played (0.0 to 100.0)
    pub songs_played: usize,    // Number of songs played since the app started
    progress_rx: Option<Receiver<(u64, u64)>>,
}

impl App {
    /// Scan the current directory for *.mp3 files
    fn new() -> io::Result<Self> {
        let files = fs::read_dir(env::current_dir()?)?
            .filter_map(|entry| entry.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext.eq_ignore_ascii_case("mp3"))
                    .unwrap_or(false)
            })
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        Ok(Self {
            files,
            selected: 0,
            status: "Use ↑/↓ or j/k • Enter to select • q to quit".into(),
            current_time: 0,        // start at zero seconds
            total_time: 0,          // unknown total duration at start
            perc_played: 0.0,       // no progress yet
            songs_played: 0,        // no songs played yet
            progress_rx: None,      // initialize receiver as None
        })
    }

    fn next(&mut self) {
        if !self.files.is_empty() {
            self.selected = (self.selected + 1) % self.files.len();
        }
    }

    fn previous(&mut self) {
        if !self.files.is_empty() {
            if self.selected == 0 {
                self.selected = self.files.len() - 1;
            } else {
                self.selected -= 1;
            }
        }
    }

    pub fn select(&mut self, progress_tx: &Sender<(u64, u64)>) {
        if self.files.is_empty() {
            self.status = "No MP3 files found".into();
        } else {
            let filename = &self.files[self.selected];
            self.status = format!("Playing: {}", filename);
            let _ = play_file(filename, progress_tx.clone());
        }
    }

    pub fn set_progress_receiver(&mut self, rx: Receiver<(u64, u64)>) {
        self.progress_rx = Some(rx);
    }

    pub fn poll_progress(&mut self) {
        if let Some(rx) = &self.progress_rx {
            while let Ok((elapsed, total)) = rx.try_recv() {
                self.current_time = elapsed;
                self.total_time = total;
                self.perc_played = if total > 0 {
                    (elapsed as f32 / total as f32) * 100.0
                } else {
                    0.0
                };
            }
        }
    }
}

fn main() -> io::Result<()> {
    // Create a channel for playback progress (elapsed_secs, total_secs)
    let (progress_tx, progress_rx) = std::sync::mpsc::channel::<(u64, u64)>();

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app and give it the receiver side of the channel
    let mut app = App::new()?;
    app.set_progress_receiver(progress_rx);

    // Run the UI loop passing terminal, app, and the sender
    let result = ui_loop(&mut terminal, &mut app, progress_tx);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
