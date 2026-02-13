# rust-quiz

Terminal-based quiz application with multiplayer support, built with Rust and ratatui.

## Features

- Clean terminal interface with ratatui
- JSON-based question files with code snippets
- Keyboard navigation
- Real-time scoring and leaderboard
- Multiplayer Mode: Host quiz sessions via WebSocket
- Host Analytics: Monitor user progress, kick/ban users

## Installation

```bash
git clone https://github.com/yourusername/rust-quiz
cd rust-quiz
cargo build --release
```

## Usage

### Local Mode (Single Player)

```bash
cargo run
# Or specify a custom questions file
cargo run -- -q path/to/questions.json
```

### Hosting a Quiz Server

Start a server for multiplayer quizzes:

```bash
cargo run -- serve -q questions.json
# Or specify a custom port (default: 8712)
cargo run -- serve -q questions.json -p 9000
```

**Host Commands:**

| Command | Description |
|---------|-------------|
| `start` | Start the quiz |
| `stop` | End quiz and send results |
| `kick <username>` | Kick a user |
| `ban <username>` | Ban user (kick + IP ban) |
| `unban <ip>` | Remove an IP ban |
| `view <username>` | View specific user progress |
| `view all` | View all users (analytics) |
| `list` | List connected users |
| `list bans` | List banned IPs |
| `help` | Show available commands |
| `quit` | Shutdown server |

### Connecting as a User

Join a hosted quiz server:

```bash
cargo run -- connect -H <host-address>
# With custom port
cargo run -- connect -H <host-address> -p 9000
```

## Question File Format

Create a JSON file with an array of questions:

```json
[
  {
    "text": "What does this function return?",
    "code": "fn example() -> i32 {\n    42\n}",
    "options": ["0", "42", "Compile error", "None"],
    "correct_answer": 1
  }
]
```

- `text`: The question prompt
- `code`: Optional code snippet (can be `null`)
- `options`: Array of 4 answer choices
- `correct_answer`: Index of correct answer (0-3)

## Navigation

- Arrow keys: Select answers
- Enter: Submit answer
- Esc: Quit

## Built With

- [Rust](https://www.rust-lang.org/)
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal handling
- [tokio](https://tokio.rs/) - Async runtime
- [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite) - WebSocket
- [serde](https://serde.rs/) - JSON serialization
- [clap](https://github.com/clap-rs/clap) - CLI argument parsing

## License

MIT
