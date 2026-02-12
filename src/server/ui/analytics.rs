//! Analytics view for the server.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::server::state::{ServerState, UserStatus};

/// Render the analytics view.
pub fn render(frame: &mut Frame, area: Rect, state: &ServerState) {
    let chunks = Layout::vertical([
        Constraint::Percentage(50), // User progress
        Constraint::Percentage(50), // Live answers
    ])
    .margin(1)
    .split(area);

    render_user_progress(frame, chunks[0], state);
    render_live_answers(frame, chunks[1], state);
}

fn render_user_progress(frame: &mut Frame, area: Rect, state: &ServerState) {
    let mut lines: Vec<Line> = Vec::new();

    let mut users: Vec<_> = state
        .sessions
        .values()
        .filter(|s| s.username.is_some())
        .collect();

    // Sort: finished first (by score desc), then in-progress (by question index desc)
    users.sort_by(|a, b| match (&a.status, &b.status) {
        (UserStatus::Finished, UserStatus::Finished) => {
            b.score.unwrap_or(0).cmp(&a.score.unwrap_or(0))
        }
        (UserStatus::Finished, _) => std::cmp::Ordering::Less,
        (_, UserStatus::Finished) => std::cmp::Ordering::Greater,
        (UserStatus::Answering(ai), UserStatus::Answering(bi)) => bi.cmp(ai),
        _ => std::cmp::Ordering::Equal,
    });

    for user in users {
        let username = user.username.as_deref().unwrap_or("???");
        let total = state.questions.len();

        match user.status {
            UserStatus::Finished => {
                let score = user.score.unwrap_or(0);
                let pct = if total > 0 {
                    (score as f64 / total as f64) * 100.0
                } else {
                    0.0
                };

                lines.push(Line::from(vec![
                    Span::styled("  + ", Style::default().fg(Color::Green)),
                    Span::styled(
                        format!("{:<14}", username),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled("[DONE]   ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("Score: {}/{} ({:.0}%)", score, total, pct),
                        Style::default().fg(Color::Green),
                    ),
                ]));
            }
            UserStatus::Answering(index) => {
                let progress = index;
                let pct = if total > 0 {
                    (progress as f64 / total as f64) * 100.0
                } else {
                    0.0
                };

                // Progress bar
                let bar_width = 15;
                let filled = ((pct / 100.0) * bar_width as f64) as usize;
                let empty = bar_width - filled;
                let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

                lines.push(Line::from(vec![
                    Span::styled("  * ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{:<14}", username),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!("[Q {:>2}/{}] ", progress + 1, total),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(bar, Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!(" {:>3.0}%", pct),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }
            UserStatus::Disconnected => {
                lines.push(Line::from(vec![
                    Span::styled("  x ", Style::default().fg(Color::Red)),
                    Span::styled(
                        format!("{:<14}", username),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled("[DISCONNECTED]", Style::default().fg(Color::Red)),
                ]));
            }
            _ => {}
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No users in quiz yet...",
            Style::default().fg(Color::DarkGray).italic(),
        )));
    }

    let widget = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" User Progress ")
            .title_style(Style::default().fg(Color::Cyan))
            .padding(Padding::horizontal(1)),
    );

    frame.render_widget(widget, area);
}

fn render_live_answers(frame: &mut Frame, area: Rect, state: &ServerState) {
    let mut lines: Vec<Line> = Vec::new();

    // Show last N answers (most recent first)
    let max_display = (area.height as usize).saturating_sub(4);
    let answers: Vec<_> = state.live_answers.iter().rev().take(max_display).collect();

    for answer in answers {
        let question = state.questions.get(answer.question_index);
        let is_correct = question.is_some_and(|q| q.correct_answer == answer.answer);

        let (symbol, color) = if is_correct {
            ("+", Color::Green)
        } else {
            ("-", Color::Red)
        };

        let option_letter = match answer.answer {
            0 => "A",
            1 => "B",
            2 => "C",
            3 => "D",
            _ => "?",
        };

        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", symbol), Style::default().fg(color)),
            Span::styled(
                format!("Q{:<3}", answer.question_index + 1),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{:<14}", answer.username),
                Style::default().fg(Color::White),
            ),
            Span::styled(" -> ", Style::default().fg(Color::DarkGray)),
            Span::styled(option_letter, Style::default().fg(color)),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Waiting for answers...",
            Style::default().fg(Color::DarkGray).italic(),
        )));
    }

    let widget = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Live Answers ")
            .title_style(Style::default().fg(Color::Cyan))
            .padding(Padding::horizontal(1)),
    );

    frame.render_widget(widget, area);
}
