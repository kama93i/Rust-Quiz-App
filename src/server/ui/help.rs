//! Help view for the server.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

/// Render the help view.
pub fn render(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "AVAILABLE COMMANDS",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  start          ", Style::default().fg(Color::Yellow)),
            Span::raw("Start the quiz (lobby only)"),
        ]),
        Line::from(vec![
            Span::styled("  stop           ", Style::default().fg(Color::Yellow)),
            Span::raw("End quiz, send results to finished users"),
        ]),
        Line::from(vec![
            Span::styled("  quit / exit    ", Style::default().fg(Color::Yellow)),
            Span::raw("Shutdown server"),
        ]),
        Line::from(vec![
            Span::styled("  kick <user>    ", Style::default().fg(Color::Yellow)),
            Span::raw("Disconnect a user"),
        ]),
        Line::from(vec![
            Span::styled("  ban <user>     ", Style::default().fg(Color::Yellow)),
            Span::raw("Kick and ban user's IP"),
        ]),
        Line::from(vec![
            Span::styled("  unban <ip>     ", Style::default().fg(Color::Yellow)),
            Span::raw("Remove IP from ban list"),
        ]),
        Line::from(vec![
            Span::styled("  view <user>    ", Style::default().fg(Color::Yellow)),
            Span::raw("Show detailed view of user"),
        ]),
        Line::from(vec![
            Span::styled("  view all       ", Style::default().fg(Color::Yellow)),
            Span::raw("Show all users analytics"),
        ]),
        Line::from(vec![
            Span::styled("  list           ", Style::default().fg(Color::Yellow)),
            Span::raw("List connected users"),
        ]),
        Line::from(vec![
            Span::styled("  list bans      ", Style::default().fg(Color::Yellow)),
            Span::raw("List banned IPs"),
        ]),
        Line::from(vec![
            Span::styled("  help / ?       ", Style::default().fg(Color::Yellow)),
            Span::raw("Show this help"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press Esc or Enter to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let widget = Paragraph::new(help_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Help ")
            .title_style(Style::default().fg(Color::Cyan))
            .padding(Padding::horizontal(2)),
    );

    frame.render_widget(widget, area);
}
