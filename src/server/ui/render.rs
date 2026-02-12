//! Main server UI renderer.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::server::state::{ServerState, ServerStatus, ServerView};

use super::{analytics, help, lobby, user_view};

/// Render the server UI based on current state.
pub fn render(frame: &mut Frame, state: &ServerState) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Min(10),   // Main content
        Constraint::Length(3), // Command history (last message)
        Constraint::Length(3), // Command input
    ])
    .split(area);

    render_header(frame, chunks[0], state);
    render_main_content(frame, chunks[1], state);
    render_command_history(frame, chunks[2], state);
    render_command_input(frame, chunks[3], state);
}

/// Render the header with status info.
fn render_header(frame: &mut Frame, area: Rect, state: &ServerState) {
    let status_str = match state.status {
        ServerStatus::Lobby => "Lobby",
        ServerStatus::InProgress => "In Progress",
        ServerStatus::Finished => "Finished",
    };

    let status_color = match state.status {
        ServerStatus::Lobby => Color::Yellow,
        ServerStatus::InProgress => Color::Green,
        ServerStatus::Finished => Color::Cyan,
    };

    let connected = state.connected_users().len();
    let named = state.named_user_count();
    let finished = state.finished_count();

    let header_text = format!(
        " Status: {}  |  Port: {}  |  Questions: {}  |  Connected: {} ({} named)  |  Finished: {}",
        status_str,
        state.port,
        state.questions.len(),
        connected,
        named,
        finished
    );

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(status_color).bold())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Quiz Server ")
                .title_style(Style::default().fg(Color::Cyan).bold()),
        );

    frame.render_widget(header, area);
}

/// Render the main content based on current view.
fn render_main_content(frame: &mut Frame, area: Rect, state: &ServerState) {
    match &state.current_view {
        ServerView::Lobby => lobby::render(frame, area, state),
        ServerView::Analytics => analytics::render(frame, area, state),
        ServerView::UserDetail(username) => user_view::render(frame, area, state, username),
        ServerView::Help => help::render(frame, area),
    }
}

/// Render the last command history message.
fn render_command_history(frame: &mut Frame, area: Rect, state: &ServerState) {
    let last_msg = state
        .command_history
        .last()
        .map(|s| s.as_str())
        .unwrap_or("");

    let history = Paragraph::new(last_msg)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));

    frame.render_widget(history, area);
}

/// Render the command input bar.
fn render_command_input(frame: &mut Frame, area: Rect, state: &ServerState) {
    let input_text = format!("> {}", state.command_input);

    let input = Paragraph::new(input_text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(input, area);

    // Show cursor position
    let cursor_x = area.x + 3 + state.command_input.len() as u16;
    let cursor_y = area.y + 1;
    frame.set_cursor_position(Position::new(cursor_x, cursor_y));
}
