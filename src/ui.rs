// ============================================================================
// em(π)trio MP3 Player — ui.rs
// Handles the main event and rendering loop for the terminal UI,
// managing playback progress, user input, and screen updates.
// Author: Tom Papatolis
// Email: tom@tpapatolis.com
// Github: https://github.com/tomgineer/empitrio
// ---------------------------------------------------------------------------
// Uses crossterm and ratatui crates to build a responsive TUI for MP3 playback.
// ============================================================================

use std::io;
use std::time::{Instant, Duration};

use crossterm::event::{self, Event as CEvent, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
    style::{Modifier, Style},
    Terminal,
};

use crate::App;
use crate::player;
use crate::theme::Theme;

/// Main event/render loop
pub fn ui_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    progress_tx: std::sync::mpsc::Sender<(u64, u64)>,
) -> io::Result<()> {
    let theme = Theme::xcad();
    let mut song_end_instant: Option<Instant> = None;

    loop {
        // Update playback progress from the channel
        app.poll_progress();

        // Auto-play next song when current song finishes
        if app.total_time == 0 || app.current_time < app.total_time || player::is_paused() {
            // Reset timer if song not finished or paused
            song_end_instant = None;
        } else {
            match song_end_instant {
                None => {
                    song_end_instant = Some(Instant::now());
                }
                Some(start) if start.elapsed() > Duration::from_millis(700) => {
                    if app.next_mp3() {
                        app.select(&progress_tx);
                    }
                    song_end_instant = None;
                }
                _ => {}
            }
        }

        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // TopBar
                    Constraint::Min(2),    // File list
                    Constraint::Length(2), // Help box (new)
                    Constraint::Length(3), // Progress bar
                    Constraint::Length(1), // Status bar
                ].as_ref())
                .split(size);

            // Top Bar
            let top_text = Paragraph::new(" e m p i t r i o — by @tomgineer {https://github.com/tomgineer/empitrio}")
                .style(Style::default().fg(theme.title));
            f.render_widget(top_text, chunks[0]);

            // --- File list widget ---
            let items: Vec<ListItem> = app.files.iter().map(|f| {
                ListItem::new(f.as_str())
                    .style(Style::default().fg(theme.text))
            }).collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title("┤   File List ├")
                        .title_style(Style::default().fg(theme.block_text))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border))
                        .style(Style::default())
                )
                .highlight_symbol("▶ ")
                .highlight_style(
                    Style::default()
                        .fg(theme.selection_text)
                        .bg(theme.selection_background)
                        .add_modifier(Modifier::BOLD),
                );

            let mut state = ListState::default();
            state.select(Some(app.selected));
            f.render_stateful_widget(list, chunks[1], &mut state);

            // --- Help Box ---
            let help_text = Paragraph::new("Help: q - Quit | p/Space - Pause/Play | ↑/↓ or j/k - Navigate | Enter - Play")
                .style(Style::default().fg(theme.text));
            f.render_widget(help_text, chunks[2]);

            // --- Progress bar ---
            let progress_label = if app.total_time == 0 {
                // Unknown duration
                format!("┤  Progress: --:-- / --:-- ├")
            } else {
                let current_time = format!("{:02}:{:02}", app.current_time / 60, app.current_time % 60);
                let total_time = format!("{:02}:{:02}", app.total_time / 60, app.total_time % 60);
                format!("┤  Progress: {} / {} ├", current_time, total_time)
            };

            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .title(progress_label)
                        .title_style(Style::default().fg(theme.block_text))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border))
                )
                .gauge_style(Style::default().fg(theme.selection_background))
                .ratio(app.perc_played as f64 / 100.0);

            f.render_widget(gauge, chunks[3]);

            // --- Status bar ---
            let status = Paragraph::new(app.status.as_str())
                .style(Style::default().fg(theme.status_text));
            f.render_widget(status, chunks[4]);
        })?;

        if event::poll(Duration::from_millis(250))? {
            if let CEvent::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('p') | KeyCode::Char(' ') => app.pause(),
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Enter => app.select(&progress_tx),
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
