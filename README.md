# em(π)trio
**Empitrio** (`emπtrio`) is a fast, minimal terminal-based MP3 player written in Rust.

It provides a simple user interface that lists MP3 files in the current directory, allows navigation with the keyboard, and plays audio using native performance.

Inspired by the elegance of the Greek letter π and the balance of a musical trio, Empitrio is designed for users who enjoy working in the terminal and want a focused music player without distractions.

## Features

- Terminal UI with a scrollable list of MP3 files
- Arrow key navigation and Enter to play
- Built with `ratatui` and `crossterm` for cross-platform TUI
- Audio playback using the `rodio` crate
- Cross-platform support (Windows, Linux, macOS)

## Requirements

- Rust (latest stable version)
- A terminal that supports ANSI escape codes (Windows Terminal, iTerm2, GNOME Terminal, etc.)

## Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/tomgineer/empitrio.git
   cd empitrio

