//! Lobby waiting screen for the client.

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::client::state::{ClientApp, ClientState};

/// Render the lobby screen.
pub fn render(frame: &mut Frame, area: Rect, app: &ClientApp) {
    let ClientState::Lobby { username } = &app.state else {
        return;
    };

    let chunks = Layout::vertical([
        Constraint::Percentage(35),
        Constraint::Length(11),
        Constraint::Percentage(35),
    ])
    .split(area);

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "RUST QUIZ",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Welcome, ", Style::default().fg(Color::White)),
            Span::styled(username, Style::default().fg(Color::Green).bold()),
            Span::styled("!", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Waiting for host to start...",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "[Q] to quit",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
    ];

    let widget = Paragraph::new(content).alignment(Alignment::Center);
    frame.render_widget(widget, chunks[1]);
}
