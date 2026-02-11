use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Padding, Paragraph},
};

use crate::app::App;

const QUESTION_PREVIEW_LENGTH: usize = 55;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let score = app.calculate_score();
    let total = app.total_questions();
    let percentage = calculate_percentage(score, total);
    let grade_color = get_grade_color(percentage);

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(6),
        Constraint::Fill(1),
        Constraint::Length(2),
    ])
    .margin(1)
    .split(area);

    render_score_summary(frame, chunks[1], score, total, percentage, grade_color);
    render_question_breakdown(frame, chunks[2], app, app.result_scroll());
    render_controls(frame, chunks[3]);
}

fn calculate_percentage(score: usize, total: usize) -> f64 {
    if total > 0 {
        (score as f64 / total as f64) * 100.0
    } else {
        0.0
    }
}

fn get_grade_color(percentage: f64) -> Color {
    match percentage as u32 {
        90..=100 => Color::Green,
        70..=89 => Color::Cyan,
        50..=69 => Color::Yellow,
        _ => Color::Red,
    }
}

fn render_score_summary(
    frame: &mut Frame,
    area: Rect,
    score: usize,
    total: usize,
    percentage: f64,
    grade_color: Color,
) {
    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "RESULTS",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("{} / {}  ({:.0}%)", score, total, percentage),
            Style::default().fg(grade_color).bold(),
        )),
        Line::from(""),
    ];

    let widget = Paragraph::new(content).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Color::DarkGray),
    );
    frame.render_widget(widget, area);
}

fn render_question_breakdown(frame: &mut Frame, area: Rect, app: &App, scroll: usize) {
    let lines: Vec<Line> = app
        .answers()
        .iter()
        .zip(app.questions().iter())
        .enumerate()
        .map(|(index, (answer, question))| {
            let is_correct = *answer == Some(question.correct_answer);
            let (symbol, color) = if is_correct {
                ("+", Color::Green)
            } else {
                ("-", Color::Red)
            };

            let preview = truncate_question(&question.text);

            Line::from(vec![
                Span::styled(format!(" {} ", symbol), Style::default().fg(color)),
                Span::styled(
                    format!("{:2}. ", index + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(preview, Style::default().fg(Color::Gray)),
            ])
        })
        .collect();

    let widget = Paragraph::new(lines)
        .block(Block::default().padding(Padding::horizontal(1)))
        .scroll((scroll as u16, 0));
    frame.render_widget(widget, area);
}

fn truncate_question(text: &str) -> String {
    let char_count = text.chars().count();
    if char_count > QUESTION_PREVIEW_LENGTH {
        let truncated: String = text.chars().take(QUESTION_PREVIEW_LENGTH).collect();
        format!("{}...", truncated)
    } else {
        text.to_string()
    }
}

fn render_controls(frame: &mut Frame, area: Rect) {
    let widget = Paragraph::new("j/k scroll  ·  r restart  ·  q quit")
        .alignment(Alignment::Center)
        .fg(Color::DarkGray);
    frame.render_widget(widget, area);
}
