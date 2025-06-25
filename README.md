
# Rust TUI Task Manager

A terminal user interface (TUI) task manager built with Rust and tui-rs, designed to manage tasks efficiently from your terminal.

## Features

- Fully keyboard-driven task management with arrow key navigation.
- Add, edit, delete, and toggle task status (pending, done, working).
- Inline editing with input boxes inside the terminal UI.
- Export tasks to Markdown and JSON files.
- Git TCR (Test-Commit-Revert) integration:
  - Run tests and auto-commit changes if tests pass.
  - Customizable test command.
- Saves tasks in a human-readable Markdown file.
- Clean and intuitive TUI inspired by `htop`.

## Advantages

- **Reliability:** Rust’s compile-time checks minimize bugs and crashes.
- **Performance:** Native execution ensures fast and responsive UI.
- **Portability:** Runs on any machine with Rust and terminal support.
- **Integration:** Built-in Git and test automation streamline your workflow.
- **User Experience:** Keyboard-centric with smooth navigation and inline editing.

## Getting Started

1. Clone this repository.
2. Install Rust and Cargo if you haven't already.
3. Run `cargo run` to start the app.
4. Use keyboard shortcuts to manage your tasks (`a` to add, `e` to edit, `d` to delete, `t` to test + commit, `E` to export).

---

Feel free to contribute or report issues!

