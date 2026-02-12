use std::fs;
use std::path::{Path, PathBuf};

use crate::models::Question;

pub fn load_questions(path: Option<&Path>) -> Vec<Question> {
    let file_path: PathBuf = match path {
        Some(p) => p.to_path_buf(),
        None => PathBuf::from("questions.json"),
    };

    load_questions_from_path(&file_path)
}

pub fn load_questions_from_path<P: AsRef<Path>>(path: P) -> Vec<Question> {
    let path = path.as_ref();

    let json_content = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Failed to read {}: {}", path.display(), err));

    let questions: Vec<Question> = serde_json::from_str(&json_content)
        .unwrap_or_else(|err| panic!("Failed to parse {}: {}", path.display(), err));

    if questions.is_empty() {
        panic!("{} must contain at least one question", path.display());
    }

    questions
}
