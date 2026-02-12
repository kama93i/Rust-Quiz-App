//! Lobby view for the server.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::server::state::{ServerState, UserStatus};

/// Render the lobby view.
pub fn render(frame: &mut Frame, area: Rect, state: &ServerState) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Title
        Constraint::Min(5),    // User list
        Constraint::Length(3), // Instructions
    ])
    .margin(1)
    .split(area);

    render_title(frame, chunks[0]);
    render_user_list(frame, chunks[1], state);
    render_instructions(frame, chunks[2], state);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("CONNECTED USERS")
        .style(Style::default().fg(Color::Cyan).bold())
        .alignment(Alignment::Center);
    frame.render_widget(title, area);
}

fn render_user_list(frame: &mut Frame, area: Rect, state: &ServerState) {
    let mut lines: Vec<Line> = Vec::new();

    // First show users with usernames
    let mut named_users: Vec<_> = state
        .sessions
        .values()
        .filter(|s| s.username.is_some() && s.is_connected())
        .collect();
    named_users.sort_by(|a, b| a.username.cmp(&b.username));

    for user in named_users {
        let username = user.username.as_deref().unwrap_or("???");
        let status = match user.status {
            UserStatus::InLobby => ("Ready", Color::Green),
            UserStatus::Answering(i) => {
                let s = format!("Q{}/{}", i + 1, state.questions.len());
                // We need to handle this differently since we can't return a String
                lines.push(Line::from(vec![
                    Span::styled("  * ", Style::default().fg(Color::Green)),
                    Span::styled(
                        format!("{:<16}", username),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!("{:<16}", user.ip_addr),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(s, Style::default().fg(Color::Yellow)),
                ]));
                continue;
            }
            UserStatus::Finished => ("Done", Color::Cyan),
            UserStatus::Disconnected => ("Disconnected", Color::Red),
            UserStatus::Connected => ("Connecting...", Color::Yellow),
        };

        lines.push(Line::from(vec![
            Span::styled("  * ", Style::default().fg(Color::Green)),
            Span::styled(
                format!("{:<16}", username),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!("{:<16}", user.ip_addr),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(status.0, Style::default().fg(status.1)),
        ]));
    }

    // Then show users without usernames (connecting)
    let unnamed_users: Vec<_> = state
        .sessions
        .values()
        .filter(|s| s.username.is_none() && s.is_connected())
        .collect();

    for user in unnamed_users {
        lines.push(Line::from(vec![
            Span::styled("  o ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:<16}", "(unnamed)"),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{:<16}", user.ip_addr),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled("Connecting...", Style::default().fg(Color::Yellow)),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No users connected yet...",
            Style::default().fg(Color::DarkGray).italic(),
        )));
    }

    let list = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .padding(Padding::horizontal(1)),
    );

    frame.render_widget(list, area);
}

fn render_instructions(frame: &mut Frame, area: Rect, state: &ServerState) {
    let text = if state.named_user_count() > 0 {
        "Type 'start' to begin the quiz  |  'help' for commands"
    } else {
        "Waiting for users to connect...  |  'help' for commands"
    };

    let instructions = Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    frame.render_widget(instructions, area);
}
