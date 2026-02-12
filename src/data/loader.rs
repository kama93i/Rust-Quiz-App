use std::fs;
use std::path::Path;

use crate::models::Question;

/// Error type for loading questions.
#[derive(Debug)]
pub enum LoadError {
    /// Failed to read the file.
    Io(std::io::Error),
    /// Failed to parse the JSON.
    Parse(serde_json::Error),
    /// The questions file is empty.
    Empty,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io(e) => write!(f, "Failed to read file: {}", e),
            LoadError::Parse(e) => write!(f, "Failed to parse JSON: {}", e),
            LoadError::Empty => write!(f, "Questions file must contain at least one question"),
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadError::Io(e) => Some(e),
            LoadError::Parse(e) => Some(e),
            LoadError::Empty => None,
        }
    }
}

impl From<std::io::Error> for LoadError {
    fn from(err: std::io::Error) -> Self {
        LoadError::Io(err)
    }
}

impl From<serde_json::Error> for LoadError {
    fn from(err: serde_json::Error) -> Self {
        LoadError::Parse(err)
    }
}

/// Load questions from a JSON file.
///
/// # Arguments
///
/// * `path` - Path to the JSON file containing questions.
///
/// # Returns
///
/// A vector of questions on success, or a `LoadError` on failure.
///
/// # Example
///
/// ```rust,no_run
/// use rust_quiz::load_questions_from_json;
///
/// let questions = load_questions_from_json("questions.json").expect("Failed to load");
/// ```
pub fn load_questions_from_json<P: AsRef<Path>>(path: P) -> Result<Vec<Question>, LoadError> {
    let json_content = fs::read_to_string(path)?;
    let questions: Vec<Question> = serde_json::from_str(&json_content)?;

    if questions.is_empty() {
        return Err(LoadError::Empty);
    }

    Ok(questions)
}
