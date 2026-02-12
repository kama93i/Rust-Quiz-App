//! Main client UI renderer.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};

use crate::client::state::{ClientApp, ClientState};

use super::{lobby, name_entry, quiz, results};

/// Render the client UI based on current state.
pub fn render(frame: &mut Frame, app: &ClientApp) {
    let area = frame.area();
    frame.render_widget(Block::default().bg(Color::Reset), area);

    match &app.state {
        ClientState::Connecting => render_connecting(frame, area, app),
        ClientState::NameEntry { .. } => name_entry::render(frame, area, app),
        ClientState::Lobby { .. } => lobby::render(frame, area, app),
        ClientState::Quiz { .. } => quiz::render(frame, area, app),
        ClientState::Results { .. } => results::render(frame, area, app),
        ClientState::Disconnected { message } => render_disconnected(frame, area, message),
    }
}

fn render_connecting(frame: &mut Frame, area: Rect, app: &ClientApp) {
    let chunks = Layout::vertical([
        Constraint::Percentage(40),
        Constraint::Length(7),
        Constraint::Percentage(40),
    ])
    .split(area);

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "RUST QUIZ",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Connecting to {}...", app.server_addr()),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
    ];

    let widget = Paragraph::new(content).alignment(Alignment::Center);
    frame.render_widget(widget, chunks[1]);
}

fn render_disconnected(frame: &mut Frame, area: Rect, message: &str) {
    let chunks = Layout::vertical([
        Constraint::Percentage(40),
        Constraint::Length(9),
        Constraint::Percentage(40),
    ])
    .split(area);

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "RUST QUIZ",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            message,
            Style::default().fg(Color::Red).bold(),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Press [Q] to exit",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
    ];

    let widget = Paragraph::new(content).alignment(Alignment::Center);
    frame.render_widget(widget, chunks[1]);
}
