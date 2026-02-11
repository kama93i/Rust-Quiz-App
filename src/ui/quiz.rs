use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
};

use crate::app::App;

const OPTION_LABELS: [char; 4] = ['A', 'B', 'C', 'D'];

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let question = app.current_question();
    let has_code = question.code.is_some();
    let chunks = create_layout(area, has_code);

    render_progress(frame, chunks[0], app);
    render_question_text(frame, chunks[1], &question.text);

    let options_chunk = if has_code {
        render_code_block(frame, chunks[2], question.code.as_ref().unwrap());
        chunks[3]
    } else {
        chunks[2]
    };

    render_options(
        frame,
        options_chunk,
        &question.options,
        app.selected_option(),
    );

    let controls_chunk = if has_code { chunks[4] } else { chunks[3] };
    render_controls(frame, controls_chunk);
}

fn create_layout(area: Rect, has_code: bool) -> std::rc::Rc<[Rect]> {
    if has_code {
        Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Min(8),
            Constraint::Length(10),
            Constraint::Length(1),
        ])
        .margin(1)
        .split(area)
    } else {
        Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(4),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .margin(2)
        .split(area)
    }
}

fn render_progress(frame: &mut Frame, area: Rect, app: &App) {
    let progress = format!(
        "{}/{}",
        app.current_question_number(),
        app.total_questions()
    );
    let widget = Paragraph::new(progress)
        .alignment(Alignment::Right)
        .fg(Color::DarkGray);
    frame.render_widget(widget, area);
}

fn render_question_text(frame: &mut Frame, area: Rect, text: &str) {
    let widget = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .fg(Color::White)
        .bold();
    frame.render_widget(widget, area);
}

fn render_code_block(frame: &mut Frame, area: Rect, code: &str) {
    let code_lines: Vec<Line> = code
        .lines()
        .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::Yellow))))
        .collect();

    let widget = Paragraph::new(code_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Color::DarkGray)
            .padding(Padding::horizontal(1)),
    );
    frame.render_widget(widget, area);
}

fn render_options(frame: &mut Frame, area: Rect, options: &[String; 4], selected: usize) {
    let mut lines: Vec<Line> = Vec::with_capacity(options.len() * 2);

    for (index, option) in options.iter().enumerate() {
        let is_selected = index == selected;
        let style = if is_selected {
            Style::default().fg(Color::Cyan).bold()
        } else {
            Style::default().fg(Color::Gray)
        };
        let marker = if is_selected { ">" } else { " " };

        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", marker), style),
            Span::styled(format!("{}. ", OPTION_LABELS[index]), style),
            Span::styled(option.as_str(), style),
        ]));
        lines.push(Line::from(""));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_controls(frame: &mut Frame, area: Rect) {
    let widget = Paragraph::new("j/k navigate  ·  enter select  ·  q quit")
        .alignment(Alignment::Center)
        .fg(Color::DarkGray);
    frame.render_widget(widget, area);
}
