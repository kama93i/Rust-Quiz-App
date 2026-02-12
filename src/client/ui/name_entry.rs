//! Name entry screen for the client.

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::client::state::{ClientApp, ClientState};

/// Render the name entry screen.
pub fn render(frame: &mut Frame, area: Rect, app: &ClientApp) {
    let ClientState::NameEntry { input, error } = &app.state else {
        return;
    };

    let chunks = Layout::vertical([
        Constraint::Percentage(35),
        Constraint::Length(11),
        Constraint::Percentage(35),
    ])
    .split(area);

    let mut content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "RUST QUIZ",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Connected to {}", app.server_addr()),
            Style::default().fg(Color::Green),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter your name: ", Style::default().fg(Color::White)),
            Span::styled(input, Style::default().fg(Color::Yellow)),
            Span::styled("_", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
    ];

    if let Some(err) = error {
        content.push(Line::from(Span::styled(
            err.clone(),
            Style::default().fg(Color::Red),
        )));
    } else {
        content.push(Line::from(""));
    }

    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        "[Enter] to join  ·  [Q] to quit",
        Style::default().fg(Color::DarkGray),
    )));

    let widget = Paragraph::new(content).alignment(Alignment::Center);
    frame.render_widget(widget, chunks[1]);
}
