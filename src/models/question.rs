use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Question {
    pub text: String,
    pub code: Option<String>,
    pub options: [String; 4],
    pub correct_answer: usize,
}
