use std::path::PathBuf;

use clap::Parser;
use rust_quiz::Quiz;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// JSON file to load the questions from
    #[arg(short, long)]
    questions: PathBuf,
}

fn main() {
    let args = Args::parse();
    let quiz = Quiz::from_json(args.questions).expect("Failed to load questions");

    if let Err(e) = quiz.run() {
        eprintln!("Error running quiz: {}", e);
        std::process::exit(1);
    }
}
