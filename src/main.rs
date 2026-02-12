use std::path::PathBuf;

use clap::{Parser, Subcommand};
use rust_quiz::protocol::DEFAULT_PORT;

#[derive(Parser)]
#[command(name = "rust-quiz")]
#[command(about = "A terminal-based quiz application with multiplayer support")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to questions JSON file (for local mode)
    #[arg(short, long, default_value = "questions.json")]
    questions: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a quiz server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value_t = DEFAULT_PORT)]
        port: u16,

        /// Path to questions JSON file
        #[arg(short, long)]
        questions: PathBuf,
    },

    /// Connect to a quiz server
    Connect {
        /// Server host address
        #[arg(short = 'H', long)]
        host: String,

        /// Server port
        #[arg(short, long, default_value_t = DEFAULT_PORT)]
        port: u16,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Serve { port, questions }) => run_server(port, questions),
        Some(Commands::Connect { host, port }) => run_client(host, port),
        None => run_local(cli.questions),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Run in local mode (single player, existing behavior).
fn run_local(questions_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use rust_quiz::Quiz;

    let quiz = Quiz::from_json(&questions_path)?;
    quiz.run()?;
    Ok(())
}

/// Run as a server host.
fn run_server(port: u16, questions_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use rust_quiz::server;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(server::run(port, questions_path))?;
    Ok(())
}

/// Run as a client connecting to a server.
fn run_client(host: String, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    use rust_quiz::client;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(client::run(host, port))?;
    Ok(())
}
