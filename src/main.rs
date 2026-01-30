use std::fs;
use std::io;
use std::panic;

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
};

use serde::Deserialize;

#[derive(Clone, Deserialize)]
struct Question {
    text: String,
    code: Option<String>,
    options: [String; 4],
    correct_answer: usize,
}

#[derive(PartialEq)]
enum AppState {
    Welcome,
    Quiz,
    Result,
}

struct App {
    state: AppState,
    questions: Vec<Question>,
    current_question: usize,
    selected_option: usize,
    answers: Vec<Option<usize>>,
}

impl App {
    fn new() -> Self {
        let questions = load_questions();
        let answers = vec![None; questions.len()];
        Self {
            state: AppState::Welcome,
            questions,
            current_question: 0,
            selected_option: 0,
            answers,
        }
    }

    fn current_question(&self) -> &Question {
        &self.questions[self.current_question]
    }

    fn select_next(&mut self) {
        self.selected_option = (self.selected_option + 1) % 4;
    }

    fn select_previous(&mut self) {
        self.selected_option = (self.selected_option + 3) % 4;
    }

    fn submit_answer(&mut self) {
        self.answers[self.current_question] = Some(self.selected_option);
        self.current_question += 1;
        self.selected_option = 0;

        if self.current_question >= self.questions.len() {
            self.state = AppState::Result;
        }
    }

    fn calculate_score(&self) -> usize {
        self.answers
            .iter()
            .zip(self.questions.iter())
            .filter(|(answer, question)| *answer == &Some(question.correct_answer))
            .count()
    }

    fn restart(&mut self) {
        self.state = AppState::Welcome;
        self.current_question = 0;
        self.selected_option = 0;
        self.answers = vec![None; self.questions.len()];
    }
}

// ============================================================================
// Questions Database
// ============================================================================

fn load_questions() -> Vec<Question> {
    let json_content = fs::read_to_string("questions.json")
        .expect("Failed to read questions.json");
    let questions: Vec<Question> = serde_json::from_str(&json_content)
        .expect("Failed to parse questions.json");
    if questions.is_empty() {
        panic!("questions.json must contain at least one question");
    }
    questions
}

// ============================================================================
// UI Rendering
// ============================================================================

fn render_welcome(frame: &mut Frame, area: Rect) {
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

fn render_quiz(frame: &mut Frame, area: Rect, app: &App) {
    let question = app.current_question();
    let has_code = question.code.is_some();

    let chunks = if has_code {
        Layout::vertical([
            Constraint::Length(1),  // Progress
            Constraint::Length(2),  // Question text
            Constraint::Min(8),     // Code block
            Constraint::Length(10), // Options
            Constraint::Length(1),  // Controls
        ])
        .margin(1)
        .split(area)
    } else {
        Layout::vertical([
            Constraint::Length(1), // Progress
            Constraint::Length(4), // Question text
            Constraint::Fill(1),   // Options
            Constraint::Length(1), // Controls
        ])
        .margin(2)
        .split(area)
    };

    // Progress
    let progress = format!("{}/{}", app.current_question + 1, app.questions.len());
    let progress_widget = Paragraph::new(progress)
        .alignment(Alignment::Right)
        .fg(Color::DarkGray);
    frame.render_widget(progress_widget, chunks[0]);

    // Question text
    let question_widget = Paragraph::new(question.text.as_str())
        .wrap(Wrap { trim: true })
        .fg(Color::White)
        .bold();
    frame.render_widget(question_widget, chunks[1]);

    // Code block (if present)
    let options_chunk = if has_code {
        let code = question.code.as_ref().unwrap();
        let code_lines: Vec<Line> = code
            .lines()
            .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::Yellow))))
            .collect();

        let code_widget = Paragraph::new(code_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Color::DarkGray)
                .padding(Padding::horizontal(1)),
        );
        frame.render_widget(code_widget, chunks[2]);
        chunks[3]
    } else {
        chunks[2]
    };

    // Options
    let option_labels = ['A', 'B', 'C', 'D'];
    let mut options_lines: Vec<Line> = Vec::new();

    for (i, option) in question.options.iter().enumerate() {
        let is_selected = i == app.selected_option;

        let style = if is_selected {
            Style::default().fg(Color::Cyan).bold()
        } else {
            Style::default().fg(Color::Gray)
        };

        let marker = if is_selected { ">" } else { " " };

        options_lines.push(Line::from(vec![
            Span::styled(format!(" {} ", marker), style),
            Span::styled(format!("{}. ", option_labels[i]), style),
            Span::styled(option.as_str(), style),
        ]));
        options_lines.push(Line::from(""));
    }

    let options_widget = Paragraph::new(options_lines);
    frame.render_widget(options_widget, options_chunk);

    // Controls
    let controls_chunk = if has_code { chunks[4] } else { chunks[3] };
    let controls = Paragraph::new("j/k navigate  ·  enter select  ·  q quit")
        .alignment(Alignment::Center)
        .fg(Color::DarkGray);
    frame.render_widget(controls, controls_chunk);
}

fn render_result(frame: &mut Frame, area: Rect, app: &App) {
    let score = app.calculate_score();
    let total = app.questions.len();
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

    let main_chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(6),
        Constraint::Fill(1),
        Constraint::Length(2),
    ])
    .margin(1)
    .split(area);

    // Score summary
    let summary = vec![
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

    let summary_widget = Paragraph::new(summary).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Color::DarkGray),
    );
    frame.render_widget(summary_widget, main_chunks[1]);

    // Question breakdown
    let mut breakdown_lines: Vec<Line> = vec![];

    for (i, (answer, question)) in app.answers.iter().zip(app.questions.iter()).enumerate() {
        let is_correct = *answer == Some(question.correct_answer);
        let (symbol, color) = if is_correct {
            ("+", Color::Green)
        } else {
            ("-", Color::Red)
        };

        let q_preview: String = question.text.chars().take(55).collect();
        let ellipsis = if question.text.chars().count() > 55 { "..." } else { "" };

        breakdown_lines.push(Line::from(vec![
            Span::styled(format!(" {} ", symbol), Style::default().fg(color)),
            Span::styled(
                format!("{:2}. ", i + 1),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{}{}", q_preview, ellipsis),
                Style::default().fg(Color::Gray),
            ),
        ]));
    }

    let breakdown_widget =
        Paragraph::new(breakdown_lines).block(Block::default().padding(Padding::horizontal(1)));
    frame.render_widget(breakdown_widget, main_chunks[2]);

    // Controls
    let controls = Paragraph::new("r restart  ·  q quit")
        .alignment(Alignment::Center)
        .fg(Color::DarkGray);
    frame.render_widget(controls, main_chunks[3]);
}

fn ui(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame.render_widget(Block::default().bg(Color::Reset), area);

    match app.state {
        AppState::Welcome => render_welcome(frame, area),
        AppState::Quiz => render_quiz(frame, area, app),
        AppState::Result => render_result(frame, area, app),
    }
}

// ============================================================================
// Main Application Loop
// ============================================================================

fn main() -> io::Result<()> {
    // Set up panic hook to restore terminal state
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = io::stdout().execute(LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut app = App::new();

    loop {
        terminal.draw(|frame| ui(frame, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match app.state {
                AppState::Welcome => match key.code {
                    KeyCode::Enter => app.state = AppState::Quiz,
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    _ => {}
                },
                AppState::Quiz => match key.code {
                    KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
                    KeyCode::Down | KeyCode::Char('j') => app.select_next(),
                    KeyCode::Enter | KeyCode::Char(' ') => app.submit_answer(),
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    _ => {}
                },
                AppState::Result => match key.code {
                    KeyCode::Char('r') | KeyCode::Char('R') => app.restart(),
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
