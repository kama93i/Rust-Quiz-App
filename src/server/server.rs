//! WebSocket server implementation.

use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message;

use crate::data::load_questions_from_json;
use crate::protocol::{validate_username, ClientMessage, ServerMessage};
use crate::terminal;

use super::commands::{execute_command, CommandResult};
use super::state::{ServerState, ServerStatus, ServerView, UserSession, UserStatus};
use super::ui;

/// Shared server state wrapped in Arc<Mutex> for async access.
type SharedState = Arc<Mutex<ServerState>>;

/// Run the quiz server.
pub async fn run<P: AsRef<Path>>(port: u16, questions_path: P) -> Result<(), Box<dyn std::error::Error>> {
    // Load questions
    let questions = load_questions_from_json(questions_path)?;
    println!("Loaded {} questions", questions.len());

    // Create shared state
    let state = Arc::new(Mutex::new(ServerState::new(questions, port)));

    // Start WebSocket server
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Server listening on {}", addr);

    // Spawn connection acceptor
    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let state = Arc::clone(&state_clone);
                    tokio::spawn(handle_connection(stream, addr, state));
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    });

    // Run TUI on main thread
    run_tui(state).await?;

    Ok(())
}

/// Handle a single WebSocket connection.
async fn handle_connection(stream: TcpStream, addr: SocketAddr, state: SharedState) {
    let ip = addr.ip();

    // Check if banned
    {
        let state_guard = state.lock().await;
        if state_guard.banned_ips.contains(&ip) {
            return;
        }
    }

    // Upgrade to WebSocket
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WebSocket handshake failed: {}", e);
            return;
        }
    };

    let (ws_sender, ws_receiver) = ws_stream.split();

    // Create channel for sending messages to this client
    let (tx, rx) = mpsc::unbounded_channel::<ServerMessage>();

    // Check for reconnection and get session_id
    let session_id = {
        let mut state_guard = state.lock().await;
        
        // First, gather info we need without holding mutable borrow
        let reconnect_info = state_guard.ip_to_id.get(&ip).copied().and_then(|existing_id| {
            let session = state_guard.sessions.get(&existing_id)?;
            if matches!(session.status, UserStatus::Disconnected) {
                let username = session.username.clone()?;
                let current_q = session.current_question_index();
                Some((existing_id, username, current_q))
            } else {
                None
            }
        });
        
        // Get status and questions info
        let server_status = state_guard.status;
        let questions_len = state_guard.questions.len();
        let question_data = if server_status == ServerStatus::InProgress {
            reconnect_info.as_ref().and_then(|(_, _, current_q)| {
                if *current_q < questions_len {
                    state_guard.questions.get(*current_q).map(|q| {
                        (*current_q, q.text.clone(), q.code.clone(), q.options.clone())
                    })
                } else {
                    None
                }
            })
        } else {
            None
        };
        
        if let Some((existing_id, username, current_q)) = reconnect_info {
            // Now do the mutable operations
            if let Some(existing) = state_guard.sessions.get_mut(&existing_id) {
                existing.sender = Some(tx.clone());
                
                // Restore status based on quiz state
                if server_status == ServerStatus::InProgress {
                    if current_q >= questions_len {
                        existing.status = UserStatus::Finished;
                    } else {
                        existing.status = UserStatus::Answering(current_q);
                    }
                } else {
                    existing.status = UserStatus::InLobby;
                }
            }
            
            state_guard.add_to_history(format!("User {} reconnected", username));
            
            // Send reconnection message
            let _ = tx.send(ServerMessage::ReconnectAccepted {
                username,
                current_question: current_q,
            });
            
            // If quiz is in progress and not finished, send current question
            if let Some((index, text, code, options)) = question_data {
                let _ = tx.send(ServerMessage::Question {
                    index,
                    text,
                    code,
                    options,
                });
            }
            
            existing_id
        } else {
            // New connection
            let session = UserSession::new(ip, tx.clone());
            let id = session.id;
            state_guard.sessions.insert(id, session);
            state_guard.ip_to_id.insert(ip, id);
            let _ = tx.send(ServerMessage::ConnectionAck);
            id
        }
    };

    // Now handle messages (lock is released)
    handle_messages(session_id, ws_sender, ws_receiver, rx, state, ip).await;
}

/// Handle messages for a connected session.
async fn handle_messages(
    session_id: uuid::Uuid,
    mut ws_sender: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<TcpStream>,
        Message,
    >,
    mut ws_receiver: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<TcpStream>,
    >,
    mut rx: mpsc::UnboundedReceiver<ServerMessage>,
    state: SharedState,
    _ip: IpAddr,
) {
    // Spawn task to forward messages from channel to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if ws_sender.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    // Process incoming messages
    while let Some(msg) = ws_receiver.next().await {
        let text = match msg {
            Ok(Message::Text(text)) => text.to_string(),
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => continue,
        };

        let client_msg: ClientMessage = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(_) => continue,
        };

        handle_client_message(session_id, client_msg, &state).await;
    }

    // Mark as disconnected
    {
        let mut state = state.lock().await;
        let username_to_log = {
            if let Some(session) = state.sessions.get_mut(&session_id) {
                session.sender = None;
                if !matches!(session.status, UserStatus::Finished) {
                    session.status = UserStatus::Disconnected;
                    session.username.clone()
                } else {
                    None
                }
            } else {
                None
            }
        };
        
        if let Some(username) = username_to_log {
            state.add_to_history(format!("User {} disconnected", username));
        }
    }

    send_task.abort();
}

/// Handle a single client message.
async fn handle_client_message(session_id: uuid::Uuid, msg: ClientMessage, state: &SharedState) {
    let mut state = state.lock().await;

    match msg {
        ClientMessage::Join { username } => {
            handle_join(session_id, username, &mut state);
        }
        ClientMessage::SubmitAnswer {
            question_index,
            answer,
        } => {
            handle_answer(session_id, question_index, answer, &mut state);
        }
    }
}

/// Handle a Join message.
fn handle_join(session_id: uuid::Uuid, username: String, state: &mut ServerState) {
    let username = username.trim().to_string();

    // Validate username
    if let Err(reason) = validate_username(&username) {
        if let Some(session) = state.sessions.get(&session_id) {
            session.send(ServerMessage::JoinRejected {
                reason: reason.to_string(),
            });
        }
        return;
    }

    // Check if username is taken
    if state.is_username_taken(&username) {
        if let Some(session) = state.sessions.get(&session_id) {
            session.send(ServerMessage::JoinRejected {
                reason: "Username is already taken".to_string(),
            });
        }
        return;
    }

    // Accept join
    if let Some(session) = state.sessions.get_mut(&session_id) {
        state.username_to_id.insert(username.clone(), session_id);
        session.username = Some(username.clone());
        
        // Set status based on quiz state
        if state.status == ServerStatus::InProgress {
            // Late joiner - start from question 0
            session.init_answers(state.questions.len());
            session.status = UserStatus::Answering(0);
            
            session.send(ServerMessage::JoinAccepted {
                username: username.clone(),
            });
            session.send(ServerMessage::QuizStart {
                total_questions: state.questions.len(),
            });
            
            // Send first question
            if let Some(q) = state.questions.first() {
                session.send(ServerMessage::Question {
                    index: 0,
                    text: q.text.clone(),
                    code: q.code.clone(),
                    options: q.options.clone(),
                });
            }
            
            state.add_to_history(format!("User {} joined (late)", username));
        } else {
            session.status = UserStatus::InLobby;
            session.send(ServerMessage::JoinAccepted {
                username: username.clone(),
            });
            state.add_to_history(format!("User {} joined", username));
        }
    }
}

/// Handle an answer submission.
fn handle_answer(
    session_id: uuid::Uuid,
    question_index: usize,
    answer: usize,
    state: &mut ServerState,
) {
    let questions_len = state.questions.len();
    let questions = state.questions.clone(); // Clone to avoid borrow issues
    
    // Get username for live answer recording
    let username = state
        .sessions
        .get(&session_id)
        .and_then(|s| s.username.clone());

    // First, update the session and collect necessary data
    let (should_finish, next_question_data, result_data) = {
        let Some(session) = state.sessions.get_mut(&session_id) else {
            return;
        };
        
        // Verify the answer is for the current question
        let current = session.current_question_index();
        if question_index != current {
            return;
        }

        // Record the answer
        if question_index < session.answers.len() {
            session.answers[question_index] = Some(answer);
        }

        // Move to next question or finish
        let next_index = question_index + 1;
        if next_index >= questions_len {
            // Quiz finished for this user
            session.status = UserStatus::Finished;
            session.finished_at = Some(Instant::now());
            session.score = Some(session.calculate_score(&questions));
            
            let score = session.score.unwrap_or(0);
            let username_for_results = session.username.clone().unwrap_or_default();
            
            // Collect answer results
            let answers: Vec<_> = session.answers
                .iter()
                .enumerate()
                .filter_map(|(i, ans)| {
                    let question = questions.get(i)?;
                    let your_answer = (*ans)?;
                    Some(crate::protocol::AnswerResult {
                        question_index: i,
                        question_text: question.text.clone(),
                        your_answer,
                        correct_answer: question.correct_answer,
                        is_correct: your_answer == question.correct_answer,
                        options: question.options.clone(),
                    })
                })
                .collect();
            
            (true, None, Some((score, username_for_results, answers)))
        } else {
            // Prepare next question
            session.status = UserStatus::Answering(next_index);
            let q_data = questions.get(next_index).map(|q| {
                (next_index, q.text.clone(), q.code.clone(), q.options.clone())
            });
            (false, q_data, None)
        }
    };

    // Record for live feed (outside the session borrow)
    if let Some(uname) = username.clone() {
        state.record_live_answer(uname, question_index, answer);
    }

    // Handle finish or send next question
    if should_finish {
        if let Some((score, username_for_results, answers)) = result_data {
            let leaderboard = state.generate_leaderboard(&username_for_results);
            
            if let Some(session) = state.sessions.get(&session_id) {
                session.send(ServerMessage::QuizResults {
                    score,
                    total: questions_len,
                    answers,
                    leaderboard,
                });
            }
            
            state.add_to_history(format!(
                "User {} finished with score {}/{}",
                username_for_results,
                score,
                questions_len
            ));
        }
    } else if let Some((index, text, code, options)) = next_question_data {
        if let Some(session) = state.sessions.get(&session_id) {
            session.send(ServerMessage::Question {
                index,
                text,
                code,
                options,
            });
        }
    }
}

/// Run the server TUI.
async fn run_tui(state: SharedState) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = terminal::init()?;

    loop {
        // Check if should quit
        {
            let state = state.lock().await;
            if state.should_quit {
                break;
            }
        }

        // Render UI
        {
            let state = state.lock().await;
            terminal.draw(|frame| ui::render(frame, &state))?;
        }

        // Handle input with timeout to allow for periodic updates
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                let should_quit = handle_input(&state, key.code).await;
                if should_quit {
                    break;
                }
            }
        }
    }

    terminal::restore()?;
    Ok(())
}

/// Handle keyboard input for the server TUI.
async fn handle_input(state: &SharedState, key: KeyCode) -> bool {
    let mut state = state.lock().await;

    // If in Help view, Esc or Enter returns to previous view
    if matches!(state.current_view, ServerView::Help) {
        if matches!(key, KeyCode::Esc | KeyCode::Enter) {
            if let Some(prev) = state.previous_view.take() {
                state.current_view = prev;
            } else {
                state.current_view = ServerView::Lobby;
            }
        }
        return false;
    }

    match key {
        KeyCode::Char(c) => {
            state.command_input.push(c);
        }
        KeyCode::Backspace => {
            state.command_input.pop();
        }
        KeyCode::Enter => {
            let input = std::mem::take(&mut state.command_input);
            let result = execute_command(&mut state, &input);

            match result {
                CommandResult::Ok(Some(msg)) => {
                    state.add_to_history(msg);
                }
                CommandResult::Ok(None) => {}
                CommandResult::Error(msg) => {
                    state.add_to_history(format!("Error: {}", msg));
                }
                CommandResult::Quit => {
                    return true;
                }
            }
        }
        KeyCode::Esc => {
            state.command_input.clear();
        }
        KeyCode::Tab => {
            // Cycle through views
            state.current_view = match state.current_view {
                ServerView::Lobby => ServerView::Analytics,
                ServerView::Analytics => ServerView::Lobby,
                ServerView::UserDetail(_) => ServerView::Analytics,
                ServerView::Help => ServerView::Lobby,
            };
        }
        _ => {}
    }

    false
}
