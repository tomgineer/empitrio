use std::fs;
use std::io::{self, BufReader};
use std::path::PathBuf;
use std::time::Duration;

use rodio::{Decoder, OutputStream, Sink};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Collect MP3 files in current directory
    let files: Vec<PathBuf> = fs::read_dir(".")?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "mp3"))
        .collect();

    if files.is_empty() {
        println!("No MP3 files found.");
        return Ok(());
    }

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut selected = 0usize;
    let mut list_state = ListState::default();
    list_state.select(Some(selected));

    // Event loop
    loop {
        terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
            .split(f.area());

            let items: Vec<ListItem> = files
                .iter()
                .map(|f| {
                    let filename = f.file_name().unwrap().to_string_lossy().to_string();
                    ListItem::new(filename)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().title("Empitrio").borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Yellow).bg(Color::Blue))
                .highlight_symbol(">> ");

            let help = Paragraph::new("↑ ↓ Navigate   ⏎ Play   q Quit")
                .style(Style::default().fg(Color::DarkGray));

            f.render_stateful_widget(list, chunks[0], &mut list_state);
            f.render_widget(help, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Down => {
                        if selected < files.len() - 1 {
                            selected += 1;
                            list_state.select(Some(selected));
                        }
                    }
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                            list_state.select(Some(selected));
                        }
                    }
                    KeyCode::Enter => {
                        play_file(&files[selected])?;
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn play_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("Playing: {}", path.display());
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    let file = std::fs::File::open(path)?;
    let source = Decoder::new(BufReader::new(file))?;
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}
