// main.rs
// Empitrio – minimal TUI that lists MP3 files and shows the chosen filename in the status bar.
// Dependencies (from Cargo.toml): rodio = "0.20", crossterm = "0.29", ratatui = "0.29"

use std::{env, fs, io};

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
    files: Vec<String>, // list of *.mp3 in current dir
    selected: usize,    // currently-highlighted index
    status: String,     // status-bar message
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

    fn select(&mut self) {
        if self.files.is_empty() {
            self.status = "No MP3 files found".into();
        } else {
            let filename = &self.files[self.selected];
            self.status = format!("Playing: {}", filename);
            let _ = play_file(filename);
        }
    }

}

fn main() -> io::Result<()> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the UI loop; capture any runtime error
    let result = ui_loop(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result // propagate potential errors
}


