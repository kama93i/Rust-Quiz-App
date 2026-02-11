mod app;
mod data;
mod models;
mod terminal;
mod ui;

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use app::App;
use models::AppState;

fn main() -> io::Result<()> {
    let mut terminal = terminal::init()?;
    let mut app = App::new();

    let result = run_event_loop(&mut terminal, &mut app);

    terminal::restore()?;
    result
}

fn run_event_loop(terminal: &mut terminal::AppTerminal, app: &mut App) -> io::Result<()> {
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
