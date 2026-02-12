//! # rust-quiz
//!
//! A terminal-based quiz application library for Rust.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use rust_quiz::{Quiz, QuizError};
//!
//! fn main() -> Result<(), QuizError> {
//!     // Load questions from a JSON file
//!     let quiz = Quiz::from_json("questions.json")?;
//!
//!     // Run the quiz in the terminal
//!     quiz.run()?;
//!
//!     Ok(())
//! }
//! ```

mod app;
mod data;
mod models;
pub mod terminal;
mod ui;

use std::io;
use std::path::Path;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

pub use app::App;
pub use data::{load_questions_from_json, LoadError};
pub use models::{AppState, Question};

/// Error type for quiz operations.
#[derive(Debug)]
pub enum QuizError {
    /// Error loading questions from file.
    Load(LoadError),
    /// IO error during quiz execution.
    Io(io::Error),
}

impl std::fmt::Display for QuizError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuizError::Load(e) => write!(f, "Failed to load questions: {}", e),
            QuizError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for QuizError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            QuizError::Load(e) => Some(e),
            QuizError::Io(e) => Some(e),
        }
    }
}

impl From<LoadError> for QuizError {
    fn from(err: LoadError) -> Self {
        QuizError::Load(err)
    }
}

impl From<io::Error> for QuizError {
    fn from(err: io::Error) -> Self {
        QuizError::Io(err)
    }
}

/// A quiz instance that can be run in the terminal.
pub struct Quiz {
    app: App,
}

impl Quiz {
    /// Create a new quiz from a vector of questions.
    pub fn new(questions: Vec<Question>) -> Self {
        Self {
            app: App::with_questions(questions),
        }
    }

    /// Load a quiz from a JSON file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON file containing questions.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rust_quiz::Quiz;
    ///
    /// let quiz = Quiz::from_json("questions.json").expect("Failed to load quiz");
    /// ```
    pub fn from_json<P: AsRef<Path>>(path: P) -> Result<Self, QuizError> {
        let questions = load_questions_from_json(path)?;
        Ok(Self::new(questions))
    }

    /// Run the quiz in the terminal.
    ///
    /// This will take over the terminal, display the quiz UI, and return
    /// when the user quits.
    pub fn run(mut self) -> Result<(), QuizError> {
        let mut term = terminal::init()?;
        let result = run_event_loop(&mut term, &mut self.app);
        terminal::restore()?;
        result
    }

    /// Get a reference to the underlying app for custom handling.
    pub fn app(&self) -> &App {
        &self.app
    }

    /// Get a mutable reference to the underlying app for custom handling.
    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }
}

fn run_event_loop(terminal: &mut terminal::AppTerminal, app: &mut App) -> Result<(), QuizError> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if handle_input(app, key.code) {
                break;
            }
        }
    }

    Ok(())
}

/// Returns true if the app should exit.
fn handle_input(app: &mut App, key: KeyCode) -> bool {
    match app.state {
        AppState::Welcome => handle_welcome_input(app, key),
        AppState::Quiz => handle_quiz_input(app, key),
        AppState::Result => handle_result_input(app, key),
    }
}

fn handle_welcome_input(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Enter => {
            app.start_quiz();
            false
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => true,
        _ => false,
    }
}

fn handle_quiz_input(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous_option();
            false
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next_option();
            false
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            app.submit_answer();
            false
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => true,
        _ => false,
    }
}

fn handle_result_input(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Down | KeyCode::Char('j') => {
            app.scroll_results_down();
            false
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.scroll_results_up();
            false
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.restart();
            false
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => true,
        _ => false,
    }
}
