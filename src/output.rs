use crate::models::{Priority, Todo};
use colored::Colorize;
use unicode_width::UnicodeWidthChar;

fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            len += UnicodeWidthChar::width(c).unwrap_or(0);
        }
    }
    len
}

fn box_width(todos: &[Todo], done_count: usize) -> usize {
    let summary_len = format!("{}/{} done", done_count, todos.len()).len();
    todos
        .iter()
        .map(|t| visible_len(&Row::from_todo(t).plain))
        .chain(["Tasks:".len(), summary_len])
        .max()
        .unwrap_or(20)
        + 2
}

struct Row {
    plain: String,
    colored: String,
}

impl Row {
    fn from_todo(todo: &Todo) -> Self {
        let status_plain = if todo.done { "[X]" } else { "[ ]" };
        let priority_plain = match todo.priority {
            Priority::High => "[!]",
            Priority::Medium => "[-]",
            Priority::Low => "   ",
        };

        let status_colored = if todo.done {
            "[X]".green().bold().to_string()
        } else {
            "[ ]".white().to_string()
        };
        let priority_colored = match todo.priority {
            Priority::High => "[!]".red().bold().to_string(),
            Priority::Medium => "[-]".yellow().to_string(),
            Priority::Low => "   ".normal().to_string(),
        };
        let text_colored = if todo.done {
            todo.text.strikethrough().dimmed().to_string()
        } else {
            todo.text.normal().to_string()
        };

        let fmt = |s: &str, p: &str, t: &str| format!("{} {} {:3}. {}", s, p, todo.id, t);

        Row {
            plain: fmt(status_plain, priority_plain, &todo.text),
            colored: fmt(&status_colored, &priority_colored, &text_colored),
        }
    }
}

fn pad(s: &str, plain: &str, width: usize) -> String {
    let spaces = width.saturating_sub(visible_len(plain));
    format!(" {}{} ", s, " ".repeat(spaces.saturating_sub(2)))
}

fn border_top(width: usize) -> String {
    format!("╔{}╗", "═".repeat(width))
}
fn border_mid(width: usize) -> String {
    format!("╠{}╣", "═".repeat(width))
}
fn border_bottom(width: usize) -> String {
    format!("╚{}╝", "═".repeat(width))
}

fn print_box_message(msg: &str) {
    let width = visible_len(msg) + 4;
    println!("\n{}", border_top(width));
    println!("║  {}  ║", msg.yellow());
    println!("{}\n", border_bottom(width));
}

fn print_row(todo: &Todo, width: usize) {
    let row = Row::from_todo(todo);
    println!("║{}║", pad(&row.colored, &row.plain, width));
}

pub fn print_todos(todos: &[Todo]) {
    if todos.is_empty() {
        print_box_message("Todos not found. Add new todo: todo add <task>");
        return;
    }

    let done = todos.iter().filter(|t| t.done).count();
    let width = box_width(todos, done);
    let summary = format!("{}/{} done", done, todos.len());
    let title = "Tasks:".bold().to_string();

    println!("\n{}", border_top(width));
    println!("║{}║", pad(&title, "Tasks:", width));
    println!("{}", border_mid(width));
    todos.iter().for_each(|t| print_row(t, width));
    println!("{}", border_mid(width));
    println!("║{}║", pad(&summary, &summary, width));
    println!("{}\n", border_bottom(width));
}

pub fn print_not_found(id: u32) {
    println!("{}", format!("Todo with ID {} not found", id).red());
}
