//! Server state management.
//!
//! This module contains all the state structures for managing
//! connected users, quiz progress, and server status.

use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::time::Instant;

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::models::Question;
use crate::protocol::{AnswerResult, LeaderboardEntry, ServerMessage};

/// Current status of the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerStatus {
    /// Waiting for host to start the quiz.
    Lobby,
    /// Quiz is in progress.
    InProgress,
    /// Quiz has finished.
    Finished,
}

/// Current status of a connected user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserStatus {
    /// Connected but hasn't provided a username yet.
    Connected,
    /// Has username, waiting in lobby for quiz to start.
    InLobby,
    /// Currently answering a question (index).
    Answering(usize),
    /// Completed all questions.
    Finished,
    /// Was connected but disconnected (can reconnect).
    Disconnected,
}

/// What view the host is currently seeing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerView {
    /// Lobby view showing connected users.
    Lobby,
    /// Analytics view showing all users' progress.
    Analytics,
    /// Detailed view of a specific user.
    UserDetail(String),
    /// Help view showing available commands.
    Help,
}

impl Default for ServerView {
    fn default() -> Self {
        Self::Lobby
    }
}

/// A single user session.
pub struct UserSession {
    /// Unique session ID.
    pub id: Uuid,
    /// Username (None until Join message received).
    pub username: Option<String>,
    /// Client IP address.
    pub ip_addr: IpAddr,
    /// Current status.
    pub status: UserStatus,
    /// Submitted answers (None = not answered yet).
    pub answers: Vec<Option<usize>>,
    /// Final score (calculated when finished).
    pub score: Option<usize>,
    /// When the user finished (for leaderboard ordering).
    pub finished_at: Option<Instant>,
    /// Channel to send messages to this client.
    pub sender: Option<mpsc::UnboundedSender<ServerMessage>>,
}

impl UserSession {
    /// Create a new session for a connected user.
    pub fn new(ip_addr: IpAddr, sender: mpsc::UnboundedSender<ServerMessage>) -> Self {
        Self {
            id: Uuid::new_v4(),
            username: None,
            ip_addr,
            status: UserStatus::Connected,
            answers: Vec::new(),
            score: None,
            finished_at: None,
            sender: Some(sender),
        }
    }

    /// Initialize answers vector for the quiz.
    pub fn init_answers(&mut self, num_questions: usize) {
        self.answers = vec![None; num_questions];
    }

    /// Get current question index (0-based).
    pub fn current_question_index(&self) -> usize {
        self.answers.iter().take_while(|a| a.is_some()).count()
    }

    /// Check if user has finished the quiz.
    pub fn is_finished(&self) -> bool {
        matches!(self.status, UserStatus::Finished)
    }

    /// Check if user is actively connected.
    pub fn is_connected(&self) -> bool {
        self.sender.is_some() && !matches!(self.status, UserStatus::Disconnected)
    }

    /// Send a message to this user.
    pub fn send(&self, msg: ServerMessage) -> bool {
        if let Some(sender) = &self.sender {
            sender.send(msg).is_ok()
        } else {
            false
        }
    }

    /// Calculate score based on answers and questions.
    pub fn calculate_score(&self, questions: &[Question]) -> usize {
        self.answers
            .iter()
            .zip(questions.iter())
            .filter(|(answer, question)| **answer == Some(question.correct_answer))
            .count()
    }

    /// Get the number of correct answers so far.
    pub fn correct_count(&self, questions: &[Question]) -> usize {
        self.answers
            .iter()
            .enumerate()
            .filter(|(i, answer)| {
                if let Some(ans) = answer {
                    questions.get(*i).is_some_and(|q| q.correct_answer == *ans)
                } else {
                    false
                }
            })
            .count()
    }

    /// Get the number of answered questions.
    pub fn answered_count(&self) -> usize {
        self.answers.iter().filter(|a| a.is_some()).count()
    }
}

/// A record of a recent answer for the live feed.
#[derive(Debug, Clone)]
pub struct LiveAnswer {
    pub username: String,
    pub question_index: usize,
    pub answer: usize,
    #[allow(dead_code)]
    pub timestamp: Instant,
}

/// Main server state.
pub struct ServerState {
    /// Current server status.
    pub status: ServerStatus,
    /// Loaded questions.
    pub questions: Vec<Question>,
    /// All user sessions (by session ID).
    pub sessions: HashMap<Uuid, UserSession>,
    /// Username to session ID mapping.
    pub username_to_id: HashMap<String, Uuid>,
    /// IP address to session ID mapping (for reconnection).
    pub ip_to_id: HashMap<IpAddr, Uuid>,
    /// Banned IP addresses.
    pub banned_ips: HashSet<IpAddr>,
    /// Current view for the host.
    pub current_view: ServerView,
    /// Previous view (for returning from Help).
    pub previous_view: Option<ServerView>,
    /// Current command input.
    pub command_input: String,
    /// Command history for display.
    pub command_history: Vec<String>,
    /// Recent live answers for analytics.
    pub live_answers: Vec<LiveAnswer>,
    /// Whether the server should shut down.
    pub should_quit: bool,
    /// Server port (for display).
    pub port: u16,
}

impl ServerState {
    /// Create a new server state with the given questions.
    pub fn new(questions: Vec<Question>, port: u16) -> Self {
        Self {
            status: ServerStatus::Lobby,
            questions,
            sessions: HashMap::new(),
            username_to_id: HashMap::new(),
            ip_to_id: HashMap::new(),
            banned_ips: HashSet::new(),
            current_view: ServerView::Lobby,
            previous_view: None,
            command_input: String::new(),
            command_history: Vec::new(),
            live_answers: Vec::new(),
            should_quit: false,
            port,
        }
    }

    /// Get all users with usernames (in lobby or playing).
    #[allow(dead_code)]
    pub fn named_users(&self) -> Vec<&UserSession> {
        self.sessions
            .values()
            .filter(|s| s.username.is_some())
            .collect()
    }

    /// Get all connected users (with or without username).
    pub fn connected_users(&self) -> Vec<&UserSession> {
        self.sessions
            .values()
            .filter(|s| s.is_connected())
            .collect()
    }

    /// Get count of users who have finished.
    pub fn finished_count(&self) -> usize {
        self.sessions.values().filter(|s| s.is_finished()).count()
    }

    /// Get count of users with usernames.
    pub fn named_user_count(&self) -> usize {
        self.sessions
            .values()
            .filter(|s| s.username.is_some())
            .count()
    }

    /// Check if a username is taken.
    pub fn is_username_taken(&self, username: &str) -> bool {
        self.username_to_id.contains_key(username)
    }

    /// Get a user session by username.
    pub fn get_user_by_name(&self, username: &str) -> Option<&UserSession> {
        self.username_to_id
            .get(username)
            .and_then(|id| self.sessions.get(id))
    }

    /// Get a mutable user session by username.
    pub fn get_user_by_name_mut(&mut self, username: &str) -> Option<&mut UserSession> {
        if let Some(id) = self.username_to_id.get(username).copied() {
            self.sessions.get_mut(&id)
        } else {
            None
        }
    }

    /// Get a user session by IP (for reconnection).
    #[allow(dead_code)]
    pub fn get_user_by_ip(&self, ip: &IpAddr) -> Option<&UserSession> {
        self.ip_to_id.get(ip).and_then(|id| self.sessions.get(id))
    }

    /// Get a mutable user session by IP.
    #[allow(dead_code)]
    pub fn get_user_by_ip_mut(&mut self, ip: &IpAddr) -> Option<&mut UserSession> {
        if let Some(id) = self.ip_to_id.get(ip).copied() {
            self.sessions.get_mut(&id)
        } else {
            None
        }
    }

    /// Add a live answer record.
    pub fn record_live_answer(&mut self, username: String, question_index: usize, answer: usize) {
        self.live_answers.push(LiveAnswer {
            username,
            question_index,
            answer,
            timestamp: Instant::now(),
        });

        // Keep only the last 50 answers
        if self.live_answers.len() > 50 {
            self.live_answers.remove(0);
        }
    }

    /// Generate leaderboard sorted by score (desc) then finish time (asc).
    pub fn generate_leaderboard(&self, requesting_username: &str) -> Vec<LeaderboardEntry> {
        let mut finished_users: Vec<_> = self
            .sessions
            .values()
            .filter(|s| s.is_finished() && s.username.is_some())
            .collect();

        // Sort by score descending, then by finish time ascending
        finished_users.sort_by(|a, b| {
            let score_cmp = b.score.unwrap_or(0).cmp(&a.score.unwrap_or(0));
            if score_cmp == std::cmp::Ordering::Equal {
                a.finished_at.cmp(&b.finished_at)
            } else {
                score_cmp
            }
        });

        finished_users
            .iter()
            .enumerate()
            .map(|(i, user)| LeaderboardEntry {
                rank: i + 1,
                username: user.username.clone().unwrap_or_default(),
                score: user.score.unwrap_or(0),
                total: self.questions.len(),
                is_you: user.username.as_deref() == Some(requesting_username),
            })
            .collect()
    }

    /// Generate answer results for a user.
    #[allow(dead_code)]
    pub fn generate_answer_results(&self, user: &UserSession) -> Vec<AnswerResult> {
        user.answers
            .iter()
            .enumerate()
            .filter_map(|(i, answer)| {
                let question = self.questions.get(i)?;
                let your_answer = (*answer)?;
                Some(AnswerResult {
                    question_index: i,
                    question_text: question.text.clone(),
                    your_answer,
                    correct_answer: question.correct_answer,
                    is_correct: your_answer == question.correct_answer,
                    options: question.options.clone(),
                })
            })
            .collect()
    }

    /// Broadcast a message to all connected users with usernames.
    pub fn broadcast(&self, msg: ServerMessage) {
        for session in self.sessions.values() {
            if session.username.is_some() && session.is_connected() {
                session.send(msg.clone());
            }
        }
    }

    /// Broadcast a message to all connected users (including those without usernames).
    pub fn broadcast_all(&self, msg: ServerMessage) {
        for session in self.sessions.values() {
            if session.is_connected() {
                session.send(msg.clone());
            }
        }
    }

    /// Add a message to command history.
    pub fn add_to_history(&mut self, msg: String) {
        self.command_history.push(msg);
        // Keep only the last 100 messages
        if self.command_history.len() > 100 {
            self.command_history.remove(0);
        }
    }
}
