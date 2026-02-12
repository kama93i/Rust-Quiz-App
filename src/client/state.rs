//! Client state management.

use crate::protocol::{AnswerResult, LeaderboardEntry};

/// Current state of the client.
#[derive(Debug, Clone)]
pub enum ClientState {
    /// Connecting to server.
    Connecting,

    /// Entering username.
    NameEntry {
        input: String,
        error: Option<String>,
    },

    /// Waiting in lobby for quiz to start.
    Lobby { username: String },

    /// Answering quiz questions.
    Quiz {
        username: String,
        current_question: Option<QuestionData>,
        current_index: usize,
        total: usize,
        selected_option: usize,
    },

    /// Viewing results after quiz completion.
    Results {
        score: usize,
        total: usize,
        answers: Vec<AnswerResult>,
        leaderboard: Vec<LeaderboardEntry>,
        scroll: usize,
    },

    /// Disconnected from server.
    Disconnected { message: String },
}

/// Data for the current question.
#[derive(Debug, Clone)]
pub struct QuestionData {
    #[allow(dead_code)]
    pub index: usize,
    pub text: String,
    pub code: Option<String>,
    pub options: [String; 4],
}

impl Default for ClientState {
    fn default() -> Self {
        Self::Connecting
    }
}

impl ClientState {
    /// Create a new name entry state.
    pub fn name_entry() -> Self {
        Self::NameEntry {
            input: String::new(),
            error: None,
        }
    }

    /// Create a new lobby state.
    pub fn lobby(username: String) -> Self {
        Self::Lobby { username }
    }

    /// Create a new quiz state.
    pub fn quiz(username: String, total: usize) -> Self {
        Self::Quiz {
            username,
            current_question: None,
            current_index: 0,
            total,
            selected_option: 0,
        }
    }

    /// Create a new results state.
    pub fn results(
        score: usize,
        total: usize,
        answers: Vec<AnswerResult>,
        leaderboard: Vec<LeaderboardEntry>,
    ) -> Self {
        Self::Results {
            score,
            total,
            answers,
            leaderboard,
            scroll: 0,
        }
    }

    /// Create a disconnected state.
    pub fn disconnected(message: String) -> Self {
        Self::Disconnected { message }
    }

    /// Check if in a terminal state (can quit).
    #[allow(dead_code)]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Results { .. } | Self::Disconnected { .. })
    }

    /// Get the username if available.
    pub fn username(&self) -> Option<&str> {
        match self {
            Self::Lobby { username } | Self::Quiz { username, .. } => Some(username),
            _ => None,
        }
    }
}

/// Client application state.
pub struct ClientApp {
    /// Current state.
    pub state: ClientState,
    /// Server host.
    pub host: String,
    /// Server port.
    pub port: u16,
    /// Whether the client should quit.
    pub should_quit: bool,
}

impl ClientApp {
    /// Create a new client app.
    pub fn new(host: String, port: u16) -> Self {
        Self {
            state: ClientState::Connecting,
            host,
            port,
            should_quit: false,
        }
    }

    /// Get the server address string.
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Move to name entry state.
    pub fn enter_name_entry(&mut self) {
        self.state = ClientState::name_entry();
    }

    /// Move to lobby state.
    pub fn enter_lobby(&mut self, username: String) {
        self.state = ClientState::lobby(username);
    }

    /// Move to quiz state.
    pub fn enter_quiz(&mut self, username: String, total: usize) {
        self.state = ClientState::quiz(username, total);
    }

    /// Set the current question.
    pub fn set_question(
        &mut self,
        index: usize,
        text: String,
        code: Option<String>,
        options: [String; 4],
    ) {
        if let ClientState::Quiz {
            current_question,
            current_index,
            selected_option,
            ..
        } = &mut self.state
        {
            *current_question = Some(QuestionData {
                index,
                text,
                code,
                options,
            });
            *current_index = index;
            *selected_option = 0;
        }
    }

    /// Move to results state.
    pub fn enter_results(
        &mut self,
        score: usize,
        total: usize,
        answers: Vec<AnswerResult>,
        leaderboard: Vec<LeaderboardEntry>,
    ) {
        self.state = ClientState::results(score, total, answers, leaderboard);
    }

    /// Move to disconnected state.
    pub fn disconnect(&mut self, message: String) {
        self.state = ClientState::disconnected(message);
    }

    /// Select next option in quiz.
    pub fn select_next_option(&mut self) {
        if let ClientState::Quiz {
            selected_option, ..
        } = &mut self.state
        {
            *selected_option = (*selected_option + 1) % 4;
        }
    }

    /// Select previous option in quiz.
    pub fn select_previous_option(&mut self) {
        if let ClientState::Quiz {
            selected_option, ..
        } = &mut self.state
        {
            *selected_option = (*selected_option + 3) % 4;
        }
    }

    /// Get current selected option.
    pub fn selected_option(&self) -> usize {
        if let ClientState::Quiz {
            selected_option, ..
        } = &self.state
        {
            *selected_option
        } else {
            0
        }
    }

    /// Get current question index.
    pub fn current_question_index(&self) -> usize {
        if let ClientState::Quiz { current_index, .. } = &self.state {
            *current_index
        } else {
            0
        }
    }

    /// Scroll results down.
    pub fn scroll_results_down(&mut self) {
        if let ClientState::Results {
            scroll, answers, ..
        } = &mut self.state
        {
            let max_scroll = answers.len().saturating_sub(1);
            *scroll = (*scroll + 1).min(max_scroll);
        }
    }

    /// Scroll results up.
    pub fn scroll_results_up(&mut self) {
        if let ClientState::Results { scroll, .. } = &mut self.state {
            *scroll = scroll.saturating_sub(1);
        }
    }

    /// Add a character to name input.
    pub fn name_input_push(&mut self, c: char) {
        if let ClientState::NameEntry { input, .. } = &mut self.state {
            if input.len() < 16 {
                input.push(c);
            }
        }
    }

    /// Remove a character from name input.
    pub fn name_input_pop(&mut self) {
        if let ClientState::NameEntry { input, .. } = &mut self.state {
            input.pop();
        }
    }

    /// Get name input value.
    pub fn name_input(&self) -> &str {
        if let ClientState::NameEntry { input, .. } = &self.state {
            input
        } else {
            ""
        }
    }

    /// Set name entry error.
    pub fn set_name_error(&mut self, err: String) {
        if let ClientState::NameEntry { error, .. } = &mut self.state {
            *error = Some(err);
        }
    }

    /// Clear name entry error.
    pub fn clear_name_error(&mut self) {
        if let ClientState::NameEntry { error, .. } = &mut self.state {
            *error = None;
        }
    }
}
