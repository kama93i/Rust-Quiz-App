//! Quiz screen for the client.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};

use crate::client::state::{ClientApp, ClientState};

/// Render the quiz screen.
pub fn render(frame: &mut Frame, area: Rect, app: &ClientApp) {
    let ClientState::Quiz {
        current_question,
        current_index,
        total,
        selected_option,
        ..
    } = &app.state
    else {
        return;
    };

    let Some(question) = current_question else {
        // Waiting for question
        let waiting = Paragraph::new("Waiting for question...")
            .alignment(Alignment::Center)
            .fg(Color::Yellow);
        frame.render_widget(waiting, area);
        return;
    };

    let has_code = question.code.is_some();

    let chunks = if has_code {
        Layout::vertical([
            Constraint::Length(3),  // Progress
            Constraint::Length(5),  // Question text
            Constraint::Length(10), // Code block
            Constraint::Min(8),     // Options
            Constraint::Length(2),  // Controls
        ])
        .margin(1)
        .split(area)
    } else {
        Layout::vertical([
            Constraint::Length(3), // Progress
            Constraint::Length(7), // Question text
            Constraint::Min(8),    // Options
            Constraint::Length(2), // Controls
        ])
        .margin(1)
        .split(area)
    };

    render_progress(frame, chunks[0], *current_index, *total);
    render_question_text(frame, chunks[1], &question.text);

    if has_code {
        render_code_block(frame, chunks[2], question.code.as_deref().unwrap_or(""));
        render_options(frame, chunks[3], &question.options, *selected_option);
        render_controls(frame, chunks[4]);
    } else {
        render_options(frame, chunks[2], &question.options, *selected_option);
        render_controls(frame, chunks[3]);
    }
}

fn render_progress(frame: &mut Frame, area: Rect, current: usize, total: usize) {
    let progress_text = format!("Question {} of {}", current + 1, total);

    let widget = Paragraph::new(progress_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan).bold());

    frame.render_widget(widget, area);
}

fn render_question_text(frame: &mut Frame, area: Rect, text: &str) {
    let widget = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .padding(Padding::horizontal(1)),
        );

    frame.render_widget(widget, area);
}

fn render_code_block(frame: &mut Frame, area: Rect, code: &str) {
    let widget = Paragraph::new(code)
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Code ")
                .title_style(Style::default().fg(Color::Cyan))
                .padding(Padding::horizontal(1)),
        );

    frame.render_widget(widget, area);
}

fn render_options(frame: &mut Frame, area: Rect, options: &[String; 4], selected: usize) {
    let option_labels = ['A', 'B', 'C', 'D'];

    let lines: Vec<Line> = options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let is_selected = i == selected;
            let prefix = if is_selected { "> " } else { "  " };
            let label = option_labels[i];

            let style = if is_selected {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default().fg(Color::White)
            };

            Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(format!("{}) ", label), style),
                Span::styled(opt.clone(), style),
            ])
        })
        .collect();

    let widget = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Options ")
            .title_style(Style::default().fg(Color::Cyan))
            .padding(Padding::horizontal(1)),
    );

    frame.render_widget(widget, area);
}

fn render_controls(frame: &mut Frame, area: Rect) {
    let widget = Paragraph::new("j/k or arrows to select  ·  Enter/Space to submit  ·  q quit")
        .alignment(Alignment::Center)
        .fg(Color::DarkGray);

    frame.render_widget(widget, area);
}
