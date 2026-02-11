use crate::data::load_questions;
use crate::models::{AppState, Question};

const NUM_OPTIONS: usize = 4;

pub struct App {
    pub state: AppState,
    questions: Vec<Question>,
    current_question_index: usize,
    selected_option: usize,
    answers: Vec<Option<usize>>,
    result_scroll: usize,
}

impl App {
    pub fn new() -> Self {
        let questions = load_questions();
        let num_questions = questions.len();

        Self {
            state: AppState::Welcome,
            questions,
            current_question_index: 0,
            selected_option: 0,
            answers: vec![None; num_questions],
            result_scroll: 0,
        }
    }

    pub fn current_question(&self) -> &Question {
        &self.questions[self.current_question_index]
    }

    pub fn current_question_number(&self) -> usize {
        self.current_question_index + 1
    }

    pub fn total_questions(&self) -> usize {
        self.questions.len()
    }

    pub fn selected_option(&self) -> usize {
        self.selected_option
    }

    pub fn questions(&self) -> &[Question] {
        &self.questions
    }

    pub fn answers(&self) -> &[Option<usize>] {
        &self.answers
    }

    pub fn result_scroll(&self) -> usize {
        self.result_scroll
    }

    pub fn scroll_results_down(&mut self) {
        let max_scroll = self.questions.len().saturating_sub(1);
        self.result_scroll = (self.result_scroll + 1).min(max_scroll);
    }

    pub fn scroll_results_up(&mut self) {
        self.result_scroll = self.result_scroll.saturating_sub(1);
    }

    pub fn select_next_option(&mut self) {
        self.selected_option = (self.selected_option + 1) % NUM_OPTIONS;
    }

    pub fn select_previous_option(&mut self) {
        self.selected_option = (self.selected_option + NUM_OPTIONS - 1) % NUM_OPTIONS;
    }

    pub fn start_quiz(&mut self) {
        self.state = AppState::Quiz;
    }

    pub fn submit_answer(&mut self) {
        self.answers[self.current_question_index] = Some(self.selected_option);
        self.current_question_index += 1;
        self.selected_option = 0;

        if self.current_question_index >= self.questions.len() {
            self.state = AppState::Result;
        }
    }

    pub fn calculate_score(&self) -> usize {
        self.answers
            .iter()
            .zip(self.questions.iter())
            .filter(|(answer, question)| *answer == &Some(question.correct_answer))
            .count()
    }

    pub fn restart(&mut self) {
        self.state = AppState::Welcome;
        self.current_question_index = 0;
        self.selected_option = 0;
        self.answers = vec![None; self.questions.len()];
        self.result_scroll = 0;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
