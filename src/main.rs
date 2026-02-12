use rust_quiz::Quiz;

fn main() {
    let quiz = Quiz::from_json("questions.json").expect("Failed to load questions");

    if let Err(e) = quiz.run() {
        eprintln!("Error running quiz: {}", e);
        std::process::exit(1);
    }
}
