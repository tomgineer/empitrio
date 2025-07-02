use std::io;
use std::time::Duration;

use crossterm::event::{self, Event as CEvent, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
    style::{Modifier, Style},
    Terminal,
};

use crate::App;
use crate::theme::Theme;

/// Main event/render loop
pub fn ui_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    progress_tx: std::sync::mpsc::Sender<(u64, u64)>,
) -> io::Result<()> {
    let theme = Theme::xcad();

    loop {
        // Update playback progress from the channel
        app.poll_progress();

        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(2),    // File list
                    Constraint::Length(3), // Progress bar
                    Constraint::Length(1), // Status bar
                ].as_ref())
                .split(size);

            // --- File list widget ---
            let items: Vec<ListItem> = app.files.iter().map(|f| {
                ListItem::new(f.as_str())
                    .style(Style::default().fg(theme.text))
            }).collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title("em(π)trio")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.border))
                        .style(Style::default())
                        .title_style(Style::default().fg(theme.title).add_modifier(Modifier::BOLD)),
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
            f.render_stateful_widget(list, chunks[0], &mut state);

            // --- Progress bar ---
            let progress_label = if app.total_time == 0 {
                // Unknown duration
                format!("Progress: --:-- / --:--")
            } else {
                let current_time = format!("{:02}:{:02}", app.current_time / 60, app.current_time % 60);
                let total_time = format!("{:02}:{:02}", app.total_time / 60, app.total_time % 60);
                format!("Progress: {} / {}", current_time, total_time)
            };

            let gauge = Gauge::default()
                .block(Block::default().title(progress_label).borders(Borders::ALL))
                .gauge_style(Style::default().fg(theme.title))
                .ratio(app.perc_played as f64 / 100.0);

            f.render_widget(gauge, chunks[1]);

            // --- Status bar ---
            let status = Paragraph::new(app.status.as_str())
                .style(Style::default().fg(theme.status_text));
            f.render_widget(status, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(250))? {
            if let CEvent::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
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
