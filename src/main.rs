use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::stdout;
use std::process::Command;
use std::{fs, io, time::Duration};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::Spans;
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui::{Terminal, backend::CrosstermBackend};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Status {
    Pending,
    Working,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    description: String,
    status: Status,
}

impl Task {
    fn new(description: String) -> Option<Self> {
        if description.trim().is_empty() {
            None
        } else {
            Some(Task {
                description,
                status: Status::Pending,
            })
        }
    }
}
const TASKS_FILE: &str = "tasks.md";

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<(), Box<dyn Error>> {
    let mut tasks = load_tasks();
    let mut selected = 0;
    let mut mode = "view"; // or "input" or "edit" or "test"
    let mut input = String::new();
    let mut test_command = String::from(" ");

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Min(3),
                    Constraint::Length(3),
                ])
                .split(f.size());

            let task_items: Vec<ListItem> = tasks.iter().enumerate().map(|(i, task)| {
                let prefix = match task.status {
                    Status::Done => "[done]",
                    Status::Working => "[working]",
                    Status::Pending => "[ ]",
                };
                let line = format!("{} {}", prefix, task.description);
                if i == selected {
                    ListItem::new(Spans::from(line)).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                } else {
                    ListItem::new(Spans::from(line))
                }
            }).collect();

            let tasks_list = List::new(task_items)
                .block(Block::default().title("Tasks (Enter: toggle, a: add, e: edit, d: delete, T: set test, t: test+commit, E: export, q: quit)").borders(Borders::ALL));

            f.render_widget(tasks_list, chunks[0]);

            if mode == "input" || mode == "edit" || mode == "test" {
                let title = match mode {
                    "input" => "Enter task description",
                    "edit" => "Edit task description",
                    "test" => "Enter test command (used by 't')",
                    _ => unreachable!(),
                };
                let input_widget = Paragraph::new(input.as_ref())
                    .block(Block::default().title(title).borders(Borders::ALL))
                    .style(Style::default().fg(Color::Green));
                f.render_widget(input_widget, chunks[1]);
            }
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match mode {
                    "view" => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('j') | KeyCode::Down => {
                            if selected < tasks.len().saturating_sub(1) {
                                selected += 1;
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if selected > 0 {
                                selected -= 1;
                            }
                        }
                        KeyCode::Char('d') => {
                            if !tasks.is_empty() {
                                tasks.remove(selected);
                                if selected > 0 {
                                    selected -= 1;
                                }
                                save_tasks(&tasks);
                            }
                        }
                        KeyCode::Char('a') => {
                            input.clear();
                            mode = "input";
                        }
                        KeyCode::Char('e') => {
                            if let Some(task) = tasks.get(selected) {
                                input = task.description.clone();
                                mode = "edit";
                            }
                        }
                        KeyCode::Char('T') => {
                            input = test_command.clone();
                            mode = "test";
                        }
                        KeyCode::Char('t') => {
                            disable_raw_mode()?;
                            execute!(
                                terminal.backend_mut(),
                                LeaveAlternateScreen,
                                DisableMouseCapture
                            )?;
                            if run_test_command(&test_command) {
                                save_tasks(&tasks);
                                if let Some(task) = tasks.get(selected) {
                                    let message =
                                        format!("TCR: completed task \"{}\"", task.description);
                                    commit_tasks(&message)
                                        .unwrap_or_else(|e| eprintln!("Commit failed: {e}"));
                                }
                            } else {
                                println!("Tests failed, not committing.");
                                let _ = Command::new("git").args(["restore", "."]).status();
                            }
                            println!("Press Enter to return to UI...");
                            let _ = io::stdin().read_line(&mut String::new());
                            enable_raw_mode()?;
                            execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                            let backend = CrosstermBackend::new(stdout());
                            *terminal = Terminal::new(backend)?;
                        }
                        KeyCode::Enter => {
                            if let Some(task) = tasks.get_mut(selected) {
                                task.status = match task.status {
                                    Status::Pending => Status::Done,
                                    Status::Done => Status::Working,
                                    Status::Working => Status::Pending,
                                };
                                save_tasks(&tasks);
                            }
                        }
                        KeyCode::Char('E') => {
                            export_to_json(&tasks);
                        }
                        _ => {}
                    },
                    "input" => match key.code {
                        KeyCode::Enter => {
                            if let Some(task) = Task::new(input.drain(..).collect()) {
                                tasks.push(task);
                                save_tasks(&tasks);
                            } else {
                                println!("⚠️ Task description cannot be empty.");
                            }
                        }
                        KeyCode::Esc => mode = "view",
                        KeyCode::Char(c) => input.push(c),
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        _ => {}
                    },
                    "edit" => match key.code {
                        KeyCode::Enter => {
                            if let Some(task) = tasks.get_mut(selected) {
                                if let Some(updated) = Task::new(input.drain(..).collect()) {
                                    *task = updated;
                                    save_tasks(&tasks);
                                } else {
                                    println!("⚠️ Updated description cannot be empty.");
                                }
                            }
                        }
                        KeyCode::Esc => mode = "view",
                        KeyCode::Char(c) => input.push(c),
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        _ => {}
                    },
                    "test" => match key.code {
                        KeyCode::Enter => {
                            test_command = input.drain(..).collect();
                            mode = "view";
                        }
                        KeyCode::Esc => mode = "view",
                        KeyCode::Char(c) => input.push(c),
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn load_tasks() -> Vec<Task> {
    let content = fs::read_to_string(TASKS_FILE).unwrap_or_default();
    content
        .lines()
        .filter(|line| line.trim().starts_with("- ["))
        .map(|line| {
            let status = if line.contains("- [x]") {
                Status::Done
            } else if line.contains("- [~]") {
                Status::Working
            } else {
                Status::Pending
            };
            let desc = line[5..].trim().to_string();
            Task {
                description: desc,
                status,
            }
        })
        .collect()
}

fn save_tasks(tasks: &[Task]) {
    let mut content = String::from("# 📋 Task List\n\n");

    // 🚧 Working
    let working: Vec<_> = tasks
        .iter()
        .filter(|t| t.status == Status::Working)
        .collect();
    if !working.is_empty() {
        content.push_str("## 🚧 Working\n");
        for task in working {
            content.push_str(&format!("- [~] {}\n", task.description));
        }
        content.push('\n');
    }

    // 📋 Pending
    let pending: Vec<_> = tasks
        .iter()
        .filter(|t| t.status == Status::Pending)
        .collect();
    if !pending.is_empty() {
        content.push_str("## 📋 Pending\n");
        for task in pending {
            content.push_str(&format!("- [ ] {}\n", task.description));
        }
        content.push('\n');
    }

    // ✅ Done
    let done: Vec<_> = tasks.iter().filter(|t| t.status == Status::Done).collect();
    if !done.is_empty() {
        content.push_str("## ✅ Done\n");
        for task in done {
            content.push_str(&format!("- [x] {}\n", task.description));
        }
        content.push('\n');
    }

    fs::write(TASKS_FILE, content).expect("Failed to write file");
}

fn export_to_json(tasks: &[Task]) {
    let json = serde_json::to_string_pretty(tasks).expect("Failed to serialize tasks");
    fs::write("tasks.json", json).expect("Failed to write JSON file");
}

fn run_test_command(command: &str) -> bool {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return false;
    }
    Command::new(parts[0])
        .args(&parts[1..])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn commit_tasks(message: &str) -> Result<(), String> {
    let add = Command::new("git")
        .args(["add", "-A"])
        .status()
        .map_err(|e| e.to_string())?;
    if !add.success() {
        return Err("git add failed".to_string());
    }

    let commit = Command::new("git")
        .args(["commit", "-m", message])
        .status()
        .map_err(|e| e.to_string())?;

    if !commit.success() {
        return Err("git commit failed".to_string());
    }

    Ok(())
}
