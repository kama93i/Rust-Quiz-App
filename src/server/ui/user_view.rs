//! User detail view for the server.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::server::state::{ServerState, UserStatus};

/// Render the user detail view.
pub fn render(frame: &mut Frame, area: Rect, state: &ServerState, username: &str) {
    let user = state.get_user_by_name(username);

    let Some(user) = user else {
        let not_found = Paragraph::new(format!("User '{}' not found", username))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" User View "));
        frame.render_widget(not_found, area);
        return;
    };

    let chunks = Layout::vertical([
        Constraint::Length(5), // User info header
        Constraint::Min(5),    // Answers grid
        Constraint::Length(3), // Stats
    ])
    .margin(1)
    .split(area);

    render_user_header(frame, chunks[0], state, user, username);
    render_answers_grid(frame, chunks[1], state, user);
    render_user_stats(frame, chunks[2], state, user);
}

fn render_user_header(
    frame: &mut Frame,
    area: Rect,
    state: &ServerState,
    user: &crate::server::state::UserSession,
    username: &str,
) {
    let status_str = match user.status {
        UserStatus::Connected => "Connecting...".to_string(),
        UserStatus::InLobby => "In Lobby".to_string(),
        UserStatus::Answering(i) => format!("Answering Q{}/{}", i + 1, state.questions.len()),
        UserStatus::Finished => "Finished".to_string(),
        UserStatus::Disconnected => "Disconnected".to_string(),
    };

    let status_color = match user.status {
        UserStatus::Connected | UserStatus::InLobby => Color::Yellow,
        UserStatus::Answering(_) => Color::Green,
        UserStatus::Finished => Color::Cyan,
        UserStatus::Disconnected => Color::Red,
    };

    let header_text = vec![
        Line::from(vec![
            Span::styled("  User: ", Style::default().fg(Color::DarkGray)),
            Span::styled(username, Style::default().fg(Color::White).bold()),
        ]),
        Line::from(vec![
            Span::styled("  IP:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                user.ip_addr.to_string(),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(Color::DarkGray)),
            Span::styled(status_str, Style::default().fg(status_color)),
        ]),
    ];

    let header = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(format!(" Viewing: {} ", username))
            .title_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(header, area);
}

fn render_answers_grid(
    frame: &mut Frame,
    area: Rect,
    state: &ServerState,
    user: &crate::server::state::UserSession,
) {
    let mut lines: Vec<Line> = Vec::new();
    let questions = &state.questions;

    // Show answers in a grid format (5 per row)
    let answers_per_row = 5;
    let mut row_spans: Vec<Span> = Vec::new();

    for (i, answer) in user.answers.iter().enumerate() {
        let question = questions.get(i);

        let (symbol, color) = match answer {
            Some(ans) => {
                let is_correct = question.is_some_and(|q| q.correct_answer == *ans);
                let letter = match ans {
                    0 => "A",
                    1 => "B",
                    2 => "C",
                    3 => "D",
                    _ => "?",
                };
                if is_correct {
                    (format!("{} +", letter), Color::Green)
                } else {
                    (format!("{} -", letter), Color::Red)
                }
            }
            None => {
                if matches!(user.status, UserStatus::Answering(idx) if idx == i) {
                    ("...".to_string(), Color::Yellow)
                } else {
                    ("---".to_string(), Color::DarkGray)
                }
            }
        };

        row_spans.push(Span::styled(
            format!("  Q{:<2}: ", i + 1),
            Style::default().fg(Color::DarkGray),
        ));
        row_spans.push(Span::styled(
            format!("{:<5}", symbol),
            Style::default().fg(color),
        ));

        if (i + 1) % answers_per_row == 0 || i == user.answers.len() - 1 {
            lines.push(Line::from(std::mem::take(&mut row_spans)));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No answers yet...",
            Style::default().fg(Color::DarkGray).italic(),
        )));
    }

    let widget = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Answers ")
            .title_style(Style::default().fg(Color::Cyan))
            .padding(Padding::horizontal(1)),
    );

    frame.render_widget(widget, area);
}

fn render_user_stats(
    frame: &mut Frame,
    area: Rect,
    state: &ServerState,
    user: &crate::server::state::UserSession,
) {
    let answered = user.answered_count();
    let correct = user.correct_count(&state.questions);
    let total = state.questions.len();

    let pct = if answered > 0 {
        (correct as f64 / answered as f64) * 100.0
    } else {
        0.0
    };

    let stats_text = format!(
        "  Progress: {}/{}  |  Correct: {}/{}  ({:.0}%)",
        answered, total, correct, answered, pct
    );

    let color = match pct as u32 {
        90..=100 => Color::Green,
        70..=89 => Color::Cyan,
        50..=69 => Color::Yellow,
        _ => Color::Red,
    };

    let stats = Paragraph::new(stats_text)
        .style(Style::default().fg(color))
        .block(Block::default().borders(Borders::TOP));

    frame.render_widget(stats, area);
}
