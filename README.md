# todo-rs

A lightweight CLI todo manager written in Rust.

## Features

- Add tasks with priority levels
- List all tasks with optional filtering
- Mark tasks as done or undone
- Remove tasks
- Colored terminal output
- Two storage backends: JSON file or SQLite database
- Configurable via TOML config file

## Installation

```bash
git clone https://github.com/status4O4/todo-rs
cd todo-rs
cargo install --path .
```

## Usage

```bash
# Add a task
todo add "Buy milk"
todo add "Fix critical bug" --priority high
todo add "Write docs" --priority medium

# List all tasks
todo list

# List only pending tasks
todo list --filter pending

# List only completed tasks
todo list --filter done

# Mark a task as done
todo done 1

# Mark a task as undone
todo undone 1

# Remove a task
todo remove 1
```

## Priority levels

| Flag | Level |
|------|-------|
| `-p low` | Low (default) |
| `-p medium` or `-p m` | Medium |
| `-p high` or `-p h` | High |

## Output

```
╔════════════════════════════════════════╗
║ Tasks:                                 ║
╠════════════════════════════════════════╣
║ [X] [!]   1. Buy milk                  ║
║ [ ] [-]   2. Write docs                ║
║ [ ] [!]   3. Fix critical bug          ║
╠════════════════════════════════════════╣
║ 1/3 done                               ║
╚════════════════════════════════════════╝
```

| Symbol | Meaning |
|--------|---------|
| `[X]` | Done |
| `[ ]` | Pending |
| `[!]` | High priority |
| `[-]` | Medium priority |

## Configuration

On first run, a config file is created at `~/.config/todo_rs/config.toml`:

```toml
storage_backend = "json"
storage_path = "/home/user/.config/todo_rs/todos.json"
database_path = "/home/user/.config/todo_rs/todos.db"
```

### Storage backends

**JSON** (default) — stores tasks in a human-readable JSON file. Easy to inspect and edit manually.

**SQLite** — stores tasks in a SQLite database. Better for large task lists.

To switch backends, change `storage_backend` in the config file:

```toml
storage_backend = "sqlite"
```

## Project structure

```
src/
├── main.rs       # CLI interface, argument parsing
├── commands.rs   # Business logic
├── models.rs     # Data structures
├── storage.rs    # Storage trait, FileStorage, SqliteStorage
├── config.rs     # Configuration loading and saving
├── error.rs      # Error types
└── output.rs     # Terminal output formatting
```

## Architecture

The project is built around a `Storage` trait that abstracts over the underlying storage backend:

```rust
pub trait Storage {
    fn load(&self) -> Result<Vec<Todo>, TodoError>;
    fn save(&self, todos: &[Todo]) -> Result<(), TodoError>;
}
```

This means `commands.rs` doesn't know or care whether data is stored in JSON or SQLite — it only talks to the trait. Switching backends requires changing one line in the config file.

## Development

```bash
# Run all tests
cargo test

# Run with arguments
cargo run -- add "Task" --priority high
cargo run -- list
cargo run -- list --filter pending
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `serde` + `serde_json` | JSON serialization |
| `rusqlite` | SQLite storage backend |
| `anyhow` | Error handling in application layer |
| `thiserror` | Error types in storage layer |
| `dirs` | Home and config directory paths |
| `colored` | Terminal colors |
| `toml` | Config file parsing |

## License

MIT
