// ui.rs
use std::io;
use std::time::Duration;

use crossterm::event::{self, Event as CEvent, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    style::{Modifier, Style},
    Terminal,
};

use crate::App;
use crate::theme::Theme;

/// Main event/render loop
pub fn ui_loop<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut app = App::new()?;
    let theme = Theme::xcad();

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(2), Constraint::Length(1)].as_ref())
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

            // --- Status bar ---
            let status = Paragraph::new(app.status.as_str())
                .style(Style::default().fg(theme.status_text));
            f.render_widget(status, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(250))? {
            if let CEvent::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Enter => app.select(),
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
