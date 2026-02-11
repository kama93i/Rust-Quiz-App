use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub fn render(frame: &mut Frame, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(9),
        Constraint::Fill(1),
    ])
    .split(area);

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "RUST QUIZ",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from(""),
        Line::from("25 Questions · Code Snippets".fg(Color::DarkGray)),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "ENTER",
            Style::default().fg(Color::Green).bold(),
        )),
        Line::from("to start".fg(Color::DarkGray)),
    ];

    let widget = Paragraph::new(content).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Color::DarkGray),
    );

    frame.render_widget(widget, chunks[1]);
}
