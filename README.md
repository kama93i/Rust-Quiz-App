# quiz-tui 📝

Terminal-based quiz application with JSON question loading, built with Rust and ratatui.

## Features

- 🖥️ Clean terminal interface
- 📄 JSON-based question files
- ⌨️ Keyboard navigation
- 📊 Real-time scoring
- 🎯 Exam simulation mode

## Installation
```bash
git clone https://github.com/yourusername/quiz-tui
cd quiz-tui
cargo build --release
./target/release/quiz-tui
```

## Usage

**1. Create a questions file (questions.json):**
```json
{
  "questions": [
    {
      "question": "What is the capital of France?",
      "options": ["London", "Berlin", "Paris", "Madrid"],
      "correct": 2
    }
  ]
}
```

**2. Run the quiz:**
```bash
quiz-tui questions.json
```

**3. Navigate:**
- Arrow keys to select answers
- Enter to submit
- ESC to quit

## Built With

- Rust
- ratatui (terminal UI)
- serde (JSON parsing)
- crossterm (terminal handling)

## License

MIT
