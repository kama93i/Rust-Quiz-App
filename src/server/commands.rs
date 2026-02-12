//! Server command parser and executor.
//!
//! Handles host commands like `start`, `kick`, `ban`, etc.

use std::net::IpAddr;

use crate::protocol::ServerMessage;

use super::state::{ServerState, ServerStatus, ServerView, UserStatus};

/// Result of executing a command.
pub enum CommandResult {
    /// Command executed successfully with optional message.
    Ok(Option<String>),
    /// Command failed with an error message.
    Error(String),
    /// Server should quit.
    Quit,
}

/// Parse and execute a command.
pub fn execute_command(state: &mut ServerState, input: &str) -> CommandResult {
    let input = input.trim();
    if input.is_empty() {
        return CommandResult::Ok(None);
    }

    let parts: Vec<&str> = input.split_whitespace().collect();
    let command = parts[0].to_lowercase();
    let args = &parts[1..];

    match command.as_str() {
        "start" => cmd_start(state),
        "stop" => cmd_stop(state),
        "quit" | "exit" => cmd_quit(state),
        "kick" => cmd_kick(state, args),
        "ban" => cmd_ban(state, args),
        "unban" => cmd_unban(state, args),
        "view" => cmd_view(state, args),
        "list" => cmd_list(state, args),
        "help" | "?" => cmd_help(),
        _ => CommandResult::Error(format!(
            "Unknown command: {}. Type 'help' for available commands.",
            command
        )),
    }
}

/// Start the quiz.
fn cmd_start(state: &mut ServerState) -> CommandResult {
    if state.status != ServerStatus::Lobby {
        return CommandResult::Error("Quiz has already started.".to_string());
    }

    let named_count = state.named_user_count();
    if named_count == 0 {
        return CommandResult::Error("No users have joined yet.".to_string());
    }

    // Initialize all users for the quiz
    let num_questions = state.questions.len();
    for session in state.sessions.values_mut() {
        if session.username.is_some() && session.status == UserStatus::InLobby {
            session.init_answers(num_questions);
            session.status = UserStatus::Answering(0);
        }
    }

    state.status = ServerStatus::InProgress;
    state.current_view = ServerView::Analytics;

    // Broadcast quiz start
    state.broadcast(ServerMessage::QuizStart {
        total_questions: num_questions,
    });

    // Send first question to each user
    if let Some(first_question) = state.questions.first() {
        let msg = ServerMessage::Question {
            index: 0,
            text: first_question.text.clone(),
            code: first_question.code.clone(),
            options: first_question.options.clone(),
        };
        state.broadcast(msg);
    }

    CommandResult::Ok(Some(format!("Quiz started with {} users!", named_count)))
}

/// Stop the quiz and send results to finished users.
fn cmd_stop(state: &mut ServerState) -> CommandResult {
    if state.status != ServerStatus::InProgress {
        return CommandResult::Error("Quiz is not in progress.".to_string());
    }

    state.status = ServerStatus::Finished;

    // Send results to all finished users, HostEndedQuiz to others
    let questions = state.questions.clone();
    let session_ids: Vec<_> = state.sessions.keys().copied().collect();

    // First pass: calculate scores and collect data
    let mut results_to_send: Vec<(
        uuid::Uuid,
        usize,
        String,
        Vec<crate::protocol::AnswerResult>,
    )> = Vec::new();
    let mut host_ended_ids: Vec<uuid::Uuid> = Vec::new();

    for id in &session_ids {
        if let Some(session) = state.sessions.get_mut(id) {
            if session.is_finished() {
                // Calculate final score
                session.score = Some(session.calculate_score(&questions));
                let username = session.username.clone().unwrap_or_default();
                let score = session.score.unwrap_or(0);

                // Collect answer results
                let answers: Vec<_> = session
                    .answers
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

                results_to_send.push((*id, score, username, answers));
            } else if session.is_connected() {
                host_ended_ids.push(*id);
            }
        }
    }

    // Second pass: send results (now we can generate leaderboards)
    for (id, score, username, answers) in results_to_send {
        let leaderboard = state.generate_leaderboard(&username);
        if let Some(session) = state.sessions.get(&id) {
            session.send(ServerMessage::QuizResults {
                score,
                total: questions.len(),
                answers,
                leaderboard,
            });
        }
    }

    // Send HostEndedQuiz to non-finished users
    for id in host_ended_ids {
        if let Some(session) = state.sessions.get(&id) {
            session.send(ServerMessage::HostEndedQuiz);
        }
    }

    CommandResult::Ok(Some(
        "Quiz stopped. Results sent to finished users.".to_string(),
    ))
}

/// Quit the server.
fn cmd_quit(state: &mut ServerState) -> CommandResult {
    // Send HostEndedQuiz to all connected users
    state.broadcast_all(ServerMessage::HostEndedQuiz);
    state.should_quit = true;
    CommandResult::Quit
}

/// Kick a user.
fn cmd_kick(state: &mut ServerState, args: &[&str]) -> CommandResult {
    if args.is_empty() {
        return CommandResult::Error("Usage: kick <username>".to_string());
    }

    let username = args[0];

    if let Some(session) = state.get_user_by_name_mut(username) {
        session.send(ServerMessage::Kicked {
            reason: "Kicked by host".to_string(),
        });
        session.sender = None;
        session.status = UserStatus::Disconnected;
        CommandResult::Ok(Some(format!("Kicked user: {}", username)))
    } else {
        CommandResult::Error(format!("User not found: {}", username))
    }
}

/// Ban a user (kick + ban IP).
fn cmd_ban(state: &mut ServerState, args: &[&str]) -> CommandResult {
    if args.is_empty() {
        return CommandResult::Error("Usage: ban <username>".to_string());
    }

    let username = args[0];

    if let Some(session) = state.get_user_by_name(username) {
        let ip = session.ip_addr;
        state.banned_ips.insert(ip);

        if let Some(session) = state.get_user_by_name_mut(username) {
            session.send(ServerMessage::Kicked {
                reason: "Banned by host".to_string(),
            });
            session.sender = None;
            session.status = UserStatus::Disconnected;
        }

        CommandResult::Ok(Some(format!("Banned user: {} (IP: {})", username, ip)))
    } else {
        CommandResult::Error(format!("User not found: {}", username))
    }
}

/// Unban an IP address.
fn cmd_unban(state: &mut ServerState, args: &[&str]) -> CommandResult {
    if args.is_empty() {
        return CommandResult::Error("Usage: unban <ip>".to_string());
    }

    let ip_str = args[0];
    match ip_str.parse::<IpAddr>() {
        Ok(ip) => {
            if state.banned_ips.remove(&ip) {
                CommandResult::Ok(Some(format!("Unbanned IP: {}", ip)))
            } else {
                CommandResult::Error(format!("IP not in ban list: {}", ip))
            }
        }
        Err(_) => CommandResult::Error(format!("Invalid IP address: {}", ip_str)),
    }
}

/// View a specific user or all users.
fn cmd_view(state: &mut ServerState, args: &[&str]) -> CommandResult {
    if args.is_empty() || args[0].to_lowercase() == "all" {
        state.current_view = ServerView::Analytics;
        CommandResult::Ok(Some("Viewing all users.".to_string()))
    } else {
        let username = args[0];
        if state.get_user_by_name(username).is_some() {
            state.current_view = ServerView::UserDetail(username.to_string());
            CommandResult::Ok(Some(format!("Viewing user: {}", username)))
        } else {
            CommandResult::Error(format!("User not found: {}", username))
        }
    }
}

/// List users or bans.
fn cmd_list(state: &mut ServerState, args: &[&str]) -> CommandResult {
    if args.first().is_some_and(|a| a.to_lowercase() == "bans") {
        if state.banned_ips.is_empty() {
            CommandResult::Ok(Some("No banned IPs.".to_string()))
        } else {
            let ips: Vec<String> = state.banned_ips.iter().map(|ip| ip.to_string()).collect();
            CommandResult::Ok(Some(format!("Banned IPs: {}", ips.join(", "))))
        }
    } else {
        let users: Vec<String> = state
            .sessions
            .values()
            .filter_map(|s| {
                let name = s.username.as_ref()?;
                let status_str = match s.status {
                    UserStatus::InLobby => "lobby".to_string(),
                    UserStatus::Answering(i) => format!("Q{}", i + 1),
                    UserStatus::Finished => "done".to_string(),
                    UserStatus::Disconnected => "disconnected".to_string(),
                    UserStatus::Connected => "connecting".to_string(),
                };
                Some(format!("{} ({})", name, status_str))
            })
            .collect();

        if users.is_empty() {
            CommandResult::Ok(Some("No users connected.".to_string()))
        } else {
            CommandResult::Ok(Some(format!("Users: {}", users.join(", "))))
        }
    }
}

/// Show help.
fn cmd_help() -> CommandResult {
    let help = r#"Available commands:
  start          - Start the quiz (lobby only)
  stop           - End quiz, send results to finished users
  quit/exit      - Shutdown server
  kick <user>    - Disconnect a user
  ban <user>     - Kick and ban user's IP
  unban <ip>     - Remove IP from ban list
  view <user>    - Show detailed view of user
  view all       - Show all users analytics
  list           - List connected users
  list bans      - List banned IPs
  help/?         - Show this help"#;
    CommandResult::Ok(Some(help.to_string()))
}
