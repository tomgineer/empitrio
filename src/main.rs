// ============================================================================
// em(π)trio MP3 Player — main.rs
// Minimal TUI listing MP3 files and showing the chosen filename in the status bar.
// Dependencies (from Cargo.toml):
//    rodio = "0.20", crossterm = "0.29", ratatui = "0.29"
// Author: Tom Papatolis
// Email: tom@tpapatolis.com
// Github: https://github.com/tomgineer/empitrio
// ---------------------------------------------------------------------------
// This is the main application entry point, defining the App state and
// handling the terminal UI lifecycle and event loop.
// ============================================================================

use std::path::PathBuf;
use std::{env, fs, io};
use std::sync::mpsc::{Receiver, Sender};

mod player;
use player::{play_file, toggle_pause, is_paused};

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
    current_dir: PathBuf,       // track current directory
    selected: usize,            // Index of the currently highlighted/selected file in the list
    status: String,             // Message shown in the status bar (e.g., "Playing", "Paused")
    pub current_time: u64,      // Elapsed playback time of the current song, in seconds
    pub total_time: u64,        // Total duration of the current song, in seconds
    pub perc_played: f32,       // Percentage of the current song played (0.0 to 100.0)
    pub songs_played: usize,    // Number of songs played since the app started
    progress_rx: Option<Receiver<(u64, u64)>>,
}

impl App {
    /// Create new App at current directory, listing folders, mp3 files and "..."
    pub fn new() -> io::Result<Self> {
        let current_dir = env::current_dir()?;
        Self::new_at_dir(current_dir)
    }

    /// Helper: Create App listing contents of a specific directory
    pub fn new_at_dir(dir: PathBuf) -> io::Result<Self> {
        let mut entries = Vec::new();

        // Add "..." entry if we can go up
        if dir.parent().is_some() {
            entries.push("...".to_string());
        }

        // List folders (with trailing /) and mp3 files
        let mut files_and_folders = fs::read_dir(&dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().into_owned();

                if path.is_dir() {
                    format!("{}/", name)
                } else if path.extension()
                    .map(|ext| ext.eq_ignore_ascii_case("mp3"))
                    .unwrap_or(false)
                {
                    name
                } else {
                    String::new()
                }
            })
            .filter(|name| !name.is_empty())
            .collect::<Vec<_>>();

        // Sort: folders first (with /), then files, both alphabetically
        files_and_folders.sort_by(|a, b| {
            let a_is_dir = a.ends_with('/');
            let b_is_dir = b.ends_with('/');
            b_is_dir.cmp(&a_is_dir).then(a.to_lowercase().cmp(&b.to_lowercase()))
        });

        entries.extend(files_and_folders);

        Ok(Self {
            files: entries,
            current_dir: dir,
            selected: 0,
            status: "Press ENTER to play or open folder...".into(),
            current_time: 0,
            total_time: 0,
            perc_played: 0.0,
            songs_played: 0,
            progress_rx: None,
        })
    }

    pub fn next(&mut self) {
        if !self.files.is_empty() {
            self.selected = (self.selected + 1) % self.files.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.files.is_empty() {
            if self.selected == 0 {
                self.selected = self.files.len() - 1;
            } else {
                self.selected -= 1;
            }
        }
    }

    /// Open folder, go up, or play file based on selection
    pub fn open_selected(&mut self, progress_tx: &Sender<(u64, u64)>) -> io::Result<()> {
        if self.files.is_empty() {
            self.status = "No files or folders found".into();
            return Ok(());
        }

        let selection = &self.files[self.selected];

        if selection == "..." {
            // Go up one directory if possible
            if let Some(parent) = self.current_dir.parent() {
                self.current_dir = parent.to_path_buf();
                *self = App::new_at_dir(self.current_dir.clone())?;
                self.status = format!("Moved up to {:?}", self.current_dir);
            } else {
                self.status = "Already at root directory".into();
            }
        } else if selection.ends_with('/') {
            // Enter folder
            let folder_name = selection.trim_end_matches('/');
            let new_path = self.current_dir.join(folder_name);
            if new_path.is_dir() {
                self.current_dir = new_path;
                *self = App::new_at_dir(self.current_dir.clone())?;
                self.status = format!("Entered folder {:?}", self.current_dir);
            } else {
                self.status = format!("Folder not found: {}", folder_name);
            }
        } else {
            // Play file
            let file_path = self.current_dir.join(selection);
            self.status = format!("Playing: {}", selection);
            let _ = play_file(file_path.to_string_lossy().as_ref(), progress_tx.clone());
        }

        Ok(())
    }

    /// Convenience: Call open_selected and update status if error
    pub fn select(&mut self, progress_tx: &Sender<(u64, u64)>) {
        if let Err(e) = self.open_selected(progress_tx) {
            self.status = format!("Error: {}", e);
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

    pub fn pause(&mut self) {
        toggle_pause();

        if is_paused() {
            self.status = "—! PAUSED !—".into();
        } else {
            if let Some(filename) = self.files.get(self.selected) {
                self.status = format!("Playing: {}", filename);
            } else {
                self.status.clear();
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
