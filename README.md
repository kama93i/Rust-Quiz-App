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
```

## Usage

**1. Create a questions file (questions.json):**
```json
[
  {
    "text": "What does this function return?",
    "code": "fn mystery() -> i32 {\n    let x = 5;\n    let y = {\n        let x = 10;\n        x\n    };\n    x + y\n}",
    "options": ["10", "15", "20", "Compile error"],
    "correct_answer": 1
  },
]
```

**2. Run the quiz:**
```bash
cargo run
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
