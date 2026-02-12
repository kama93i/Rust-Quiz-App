//! Protocol messages for client-server communication.
//!
//! All messages are serialized as JSON over WebSocket.

use serde::{Deserialize, Serialize};

/// Messages sent from client to server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Client wants to join with a username.
    Join { username: String },

    /// Client submits an answer for the current question.
    SubmitAnswer {
        question_index: usize,
        answer: usize,
    },
}

/// Messages sent from server to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Connection accepted, waiting for Join message.
    ConnectionAck,

    /// Username accepted, client is now in lobby.
    JoinAccepted { username: String },

    /// Username rejected (taken, invalid length, etc.).
    JoinRejected { reason: String },

    /// Reconnection successful, resuming previous session.
    ReconnectAccepted {
        username: String,
        current_question: usize,
    },

    /// Quiz is starting.
    QuizStart { total_questions: usize },

    /// Next question to answer.
    Question {
        index: usize,
        text: String,
        code: Option<String>,
        options: [String; 4],
    },

    /// Quiz complete with results.
    QuizResults {
        score: usize,
        total: usize,
        answers: Vec<AnswerResult>,
        leaderboard: Vec<LeaderboardEntry>,
    },

    /// Client has been kicked by host.
    Kicked { reason: String },

    /// Host ended the quiz abruptly.
    HostEndedQuiz,

    /// Server is shutting down.
    ServerClosing,
}

/// Result for a single answered question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerResult {
    pub question_index: usize,
    pub question_text: String,
    pub your_answer: usize,
    pub correct_answer: usize,
    pub is_correct: bool,
    pub options: [String; 4],
}

/// Entry in the leaderboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: usize,
    pub username: String,
    pub score: usize,
    pub total: usize,
    pub is_you: bool,
}

/// Username validation constants.
pub const USERNAME_MIN_LENGTH: usize = 3;
pub const USERNAME_MAX_LENGTH: usize = 16;

/// Default server port.
pub const DEFAULT_PORT: u16 = 8712;

/// Validates a username according to the rules.
///
/// Returns `Ok(())` if valid, or `Err` with an error message.
pub fn validate_username(username: &str) -> Result<(), &'static str> {
    let trimmed = username.trim();

    if trimmed.len() < USERNAME_MIN_LENGTH {
        return Err("Username must be at least 3 characters");
    }

    if trimmed.len() > USERNAME_MAX_LENGTH {
        return Err("Username must be at most 16 characters");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        assert!(validate_username("abc").is_ok());
        assert!(validate_username("abcdefghijklmnop").is_ok()); // 16 chars
        assert!(validate_username("ab").is_err());
        assert!(validate_username("abcdefghijklmnopq").is_err()); // 17 chars
        assert!(validate_username("  ab  ").is_err()); // trimmed = 2 chars
    }

    #[test]
    fn test_message_serialization() {
        let msg = ClientMessage::Join {
            username: "Alice".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"Join\""));

        let msg = ServerMessage::QuizStart {
            total_questions: 25,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"QuizStart\""));
    }
}
