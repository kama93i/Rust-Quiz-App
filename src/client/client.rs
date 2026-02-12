//! WebSocket client implementation.

use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message;

use crate::protocol::{ClientMessage, ServerMessage};
use crate::terminal;

use super::state::{ClientApp, ClientState};
use super::ui;

/// Shared client app state.
type SharedApp = Arc<Mutex<ClientApp>>;

/// Run the quiz client.
pub async fn run(host: String, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = Arc::new(Mutex::new(ClientApp::new(host.clone(), port)));

    // Connect to server
    let url = format!("ws://{}:{}", host, port);
    println!("Connecting to {}...", url);

    let (ws_stream, _) = match tokio_tungstenite::connect_async(&url).await {
        Ok(result) => result,
        Err(e) => {
            return Err(format!("Failed to connect to server: {}", e).into());
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Create channel for outgoing messages
    let (tx, mut rx) = mpsc::unbounded_channel::<ClientMessage>();

    // Spawn task to send messages
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if ws_sender.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    // Spawn task to receive messages
    let app_clone = Arc::clone(&app);
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            let text = match msg {
                Ok(Message::Text(text)) => text.to_string(),
                Ok(Message::Close(_)) => {
                    let mut app = app_clone.lock().await;
                    app.disconnect("Connection closed by server".to_string());
                    break;
                }
                Err(e) => {
                    let mut app = app_clone.lock().await;
                    app.disconnect(format!("Connection error: {}", e));
                    break;
                }
                _ => continue,
            };

            let server_msg: ServerMessage = match serde_json::from_str(&text) {
                Ok(m) => m,
                Err(_) => continue,
            };

            handle_server_message(&app_clone, server_msg).await;
        }
    });

    // Run TUI
    run_tui(app, tx).await?;

    // Clean up
    recv_task.abort();

    Ok(())
}

/// Handle a message from the server.
async fn handle_server_message(app: &SharedApp, msg: ServerMessage) {
    let mut app = app.lock().await;

    match msg {
        ServerMessage::ConnectionAck => {
            app.enter_name_entry();
        }
        ServerMessage::JoinAccepted { username } => {
            app.enter_lobby(username);
        }
        ServerMessage::JoinRejected { reason } => {
            app.set_name_error(reason);
        }
        ServerMessage::ReconnectAccepted {
            username,
            current_question: _,
        } => {
            // We'll receive QuizStart and Question messages separately
            // For now, just note we reconnected
            app.state = ClientState::Lobby { username };
        }
        ServerMessage::QuizStart { total_questions } => {
            let username = app.state.username().unwrap_or("").to_string();
            app.enter_quiz(username, total_questions);
        }
        ServerMessage::Question {
            index,
            text,
            code,
            options,
        } => {
            // Update quiz with new question
            if let ClientState::Quiz { .. } = &app.state {
                app.set_question(index, text, code, options);
            } else {
                // Might be reconnecting or late joining
                let username = app.state.username().unwrap_or("").to_string();
                // We don't have total here, but we can estimate
                app.state = ClientState::Quiz {
                    username,
                    current_question: Some(super::state::QuestionData {
                        index,
                        text,
                        code,
                        options,
                    }),
                    current_index: index,
                    total: index + 1, // Will be updated as we get more questions
                    selected_option: 0,
                };
            }
        }
        ServerMessage::QuizResults {
            score,
            total,
            answers,
            leaderboard,
        } => {
            app.enter_results(score, total, answers, leaderboard);
        }
        ServerMessage::Kicked { reason } => {
            app.disconnect(format!("Kicked: {}", reason));
        }
        ServerMessage::HostEndedQuiz => {
            app.disconnect("HOST ENDED QUIZ".to_string());
        }
        ServerMessage::ServerClosing => {
            app.disconnect("Server is shutting down".to_string());
        }
    }
}

/// Run the client TUI.
async fn run_tui(
    app: SharedApp,
    tx: mpsc::UnboundedSender<ClientMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = terminal::init()?;

    loop {
        // Check if should quit
        {
            let app = app.lock().await;
            if app.should_quit {
                break;
            }
        }

        // Render UI
        {
            let app = app.lock().await;
            terminal.draw(|frame| ui::render(frame, &app))?;
        }

        // Handle input with timeout
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                let should_quit = handle_input(&app, &tx, key.code).await;
                if should_quit {
                    break;
                }
            }
        }
    }

    terminal::restore()?;
    Ok(())
}

/// Handle keyboard input.
async fn handle_input(
    app: &SharedApp,
    tx: &mpsc::UnboundedSender<ClientMessage>,
    key: KeyCode,
) -> bool {
    let mut app = app.lock().await;

    match &app.state {
        ClientState::Connecting => {
            if matches!(key, KeyCode::Char('q') | KeyCode::Char('Q')) {
                app.should_quit = true;
                return true;
            }
        }
        ClientState::NameEntry { .. } => {
            match key {
                KeyCode::Char('q') | KeyCode::Char('Q') if app.name_input().is_empty() => {
                    app.should_quit = true;
                    return true;
                }
                KeyCode::Char(c) => {
                    app.clear_name_error();
                    app.name_input_push(c);
                }
                KeyCode::Backspace => {
                    app.clear_name_error();
                    app.name_input_pop();
                }
                KeyCode::Enter => {
                    let username = app.name_input().to_string();
                    if !username.is_empty() {
                        let _ = tx.send(ClientMessage::Join { username });
                    }
                }
                KeyCode::Esc => {
                    app.should_quit = true;
                    return true;
                }
                _ => {}
            }
        }
        ClientState::Lobby { .. } => {
            if matches!(key, KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc) {
                app.should_quit = true;
                return true;
            }
        }
        ClientState::Quiz { current_question, .. } => {
            match key {
                KeyCode::Up | KeyCode::Char('k') => {
                    app.select_previous_option();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.select_next_option();
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if current_question.is_some() {
                        let question_index = app.current_question_index();
                        let answer = app.selected_option();
                        let _ = tx.send(ClientMessage::SubmitAnswer {
                            question_index,
                            answer,
                        });
                    }
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    app.should_quit = true;
                    return true;
                }
                _ => {}
            }
        }
        ClientState::Results { .. } => {
            match key {
                KeyCode::Down | KeyCode::Char('j') => {
                    app.scroll_results_down();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.scroll_results_up();
                }
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    app.should_quit = true;
                    return true;
                }
                _ => {}
            }
        }
        ClientState::Disconnected { .. } => {
            if matches!(key, KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc | KeyCode::Enter) {
                app.should_quit = true;
                return true;
            }
        }
    }

    false
}
