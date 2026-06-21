mod commands;
mod config;
mod error;
mod models;
mod output;
mod storage;

use clap::{Parser, Subcommand};

use crate::models::Priority;

#[derive(Parser)]
#[command(name = "todo", about = "Light todo manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        text: String,
        #[arg(short, long, default_value = "low")]
        priority: String,
    },
    List {
        #[arg(short, long)]
        filter: Option<String>,
    },
    Done {
        id: u32,
    },
    Undone {
        id: u32,
    },
    Remove {
        id: u32,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn parse_priority(s: &str) -> Priority {
    match s.to_lowercase().as_str() {
        "medium" | "m" => Priority::Medium,
        "high" | "h" => Priority::High,
        _ => Priority::Low,
    }
}
fn run() -> anyhow::Result<()> {
    let config = config::load_config()?;
    config::save_default_config()?;

    let storage = storage::make_storage(&config)?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { text, priority } => {
            let p = parse_priority(&priority);
            let todos = commands::add_todo(storage.as_ref(), text, p)?;
            output::print_todos(&todos);
        }
        Commands::List { filter } => {
            let todos = commands::list_todos(storage.as_ref(), filter.as_deref())?;
            output::print_todos(&todos);
        }
        Commands::Done { id } => match commands::mark_done(storage.as_ref(), id)? {
            Some(todos) => output::print_todos(&todos),
            None => output::print_not_found(id),
        },
        Commands::Undone { id } => match commands::mark_undone(storage.as_ref(), id)? {
            Some(todos) => output::print_todos(&todos),
            None => output::print_not_found(id),
        },
        Commands::Remove { id } => match commands::remove_todo(storage.as_ref(), id)? {
            Some(_) => println!("Task with ID {} deleted", id),
            None => output::print_not_found(id),
        },
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_priority_high() {
        assert_eq!(parse_priority("high"), Priority::High);
        assert_eq!(parse_priority("h"), Priority::High);
        assert_eq!(parse_priority("HIGH"), Priority::High);
    }

    #[test]
    fn test_parse_priority_medium() {
        assert_eq!(parse_priority("medium"), Priority::Medium);
        assert_eq!(parse_priority("m"), Priority::Medium);
        assert_eq!(parse_priority("MEDIUM"), Priority::Medium);
    }

    #[test]
    fn test_parse_priority_default() {
        assert_eq!(parse_priority("low"), Priority::Low);
        assert_eq!(parse_priority("unknown"), Priority::Low);
        assert_eq!(parse_priority(""), Priority::Low);
    }
}
