//! Results screen for the client.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::client::state::{ClientApp, ClientState};

const QUESTION_PREVIEW_LENGTH: usize = 45;

/// Render the results screen.
pub fn render(frame: &mut Frame, area: Rect, app: &ClientApp) {
    let ClientState::Results {
        score,
        total,
        answers,
        leaderboard,
        scroll,
    } = &app.state
    else {
        return;
    };

    let chunks = Layout::vertical([
        Constraint::Length(6), // Score summary
        Constraint::Min(8),    // Answers breakdown
        Constraint::Length(8), // Leaderboard
        Constraint::Length(2), // Controls
    ])
    .margin(1)
    .split(area);

    render_score_summary(frame, chunks[0], *score, *total);
    render_answers(frame, chunks[1], answers, *scroll);
    render_leaderboard(frame, chunks[2], leaderboard);
    render_controls(frame, chunks[3]);
}

fn render_score_summary(frame: &mut Frame, area: Rect, score: usize, total: usize) {
    let percentage = if total > 0 {
        (score as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let grade_color = match percentage as u32 {
        90..=100 => Color::Green,
        70..=89 => Color::Cyan,
        50..=69 => Color::Yellow,
        _ => Color::Red,
    };

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

fn render_answers(
    frame: &mut Frame,
    area: Rect,
    answers: &[crate::protocol::AnswerResult],
    scroll: usize,
) {
    let lines: Vec<Line> = answers
        .iter()
        .enumerate()
        .map(|(index, answer)| {
            let (symbol, color) = if answer.is_correct {
                ("+", Color::Green)
            } else {
                ("-", Color::Red)
            };

            let preview = truncate_question(&answer.question_text);

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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Your Answers ")
                .title_style(Style::default().fg(Color::Cyan))
                .padding(Padding::horizontal(1)),
        )
        .scroll((scroll as u16, 0));

    frame.render_widget(widget, area);
}

fn render_leaderboard(
    frame: &mut Frame,
    area: Rect,
    leaderboard: &[crate::protocol::LeaderboardEntry],
) {
    let lines: Vec<Line> = leaderboard
        .iter()
        .take(5) // Show top 5
        .map(|entry| {
            let rank_style = match entry.rank {
                1 => Style::default().fg(Color::Yellow).bold(),
                2 => Style::default().fg(Color::White),
                3 => Style::default().fg(Color::LightRed),
                _ => Style::default().fg(Color::DarkGray),
            };

            let you_marker = if entry.is_you { " <- You" } else { "" };

            let pct = if entry.total > 0 {
                (entry.score as f64 / entry.total as f64) * 100.0
            } else {
                0.0
            };

            Line::from(vec![
                Span::styled(format!("  {}. ", entry.rank), rank_style),
                Span::styled(
                    format!("{:<14}", entry.username),
                    if entry.is_you {
                        Style::default().fg(Color::Green).bold()
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
                Span::styled(
                    format!("{}/{} ({:.0}%)", entry.score, entry.total, pct),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(you_marker, Style::default().fg(Color::Green)),
            ])
        })
        .collect();

    let widget = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Leaderboard ")
            .title_style(Style::default().fg(Color::Cyan))
            .padding(Padding::horizontal(1)),
    );

    frame.render_widget(widget, area);
}

fn render_controls(frame: &mut Frame, area: Rect) {
    let widget = Paragraph::new("j/k scroll  ·  q quit")
        .alignment(Alignment::Center)
        .fg(Color::DarkGray);

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
