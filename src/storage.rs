use crate::config::{Config, StorageBackend};
use crate::error::TodoError;
use crate::models::{Priority, Todo};
use rusqlite::{Connection, params};
use std::fs;
use std::path::PathBuf;

pub trait Storage {
    fn load(&self) -> Result<Vec<Todo>, TodoError>;
    fn save(&self, todos: &[Todo]) -> Result<(), TodoError>;
}

pub struct FileStorage {
    path: PathBuf,
}

impl FileStorage {
    pub fn new(config: &Config) -> Self {
        FileStorage {
            path: config.storage_path.clone(),
        }
    }
}

impl Storage for FileStorage {
    fn load(&self) -> Result<Vec<Todo>, TodoError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.path)?;
        let todos = serde_json::from_str(&content)?;
        Ok(todos)
    }
    fn save(&self, todos: &[Todo]) -> Result<(), TodoError> {
        let content = serde_json::to_string_pretty(todos)?;
        fs::write(&self.path, content)?;
        Ok(())
    }
}

pub struct SqliteStorage {
    path: PathBuf,
}

impl SqliteStorage {
    pub fn new(config: &Config) -> Result<Self, TodoError> {
        let storage = SqliteStorage {
            path: config.database_path.clone(),
        };
        storage.init()?;
        Ok(storage)
    }

    fn connect(&self) -> Result<Connection, TodoError> {
        Ok(Connection::open(&self.path)?)
    }

    fn init(&self) -> Result<(), TodoError> {
        let conn = self.connect()?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS todos (
                id INTEGER PRIMARY KEY,
                text TEXT NOT NULL,
                done BOOLEAN NOT NULL DEFAULT 0,
                priority TEXT NOT NULL DEFAULT 'Low'
            );
        ",
        )?;
        Ok(())
    }
}

impl Storage for SqliteStorage {
    fn load(&self) -> Result<Vec<Todo>, TodoError> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare("SELECT id, text, done, priority FROM todos ORDER BY id")?;
        let todos = stmt
            .query_map([], |row| {
                let priority_str: String = row.get(3)?;
                let priority = match priority_str.as_str() {
                    "High" => Priority::High,
                    "Medium" => Priority::Medium,
                    _ => Priority::Low,
                };
                Ok(Todo {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    done: row.get(2)?,
                    priority,
                })
            })?
            .collect::<Result<Vec<Todo>, _>>()?;
        Ok(todos)
    }

    fn save(&self, todos: &[Todo]) -> Result<(), TodoError> {
        let conn = self.connect()?;
        conn.execute_batch("DELETE FROM todos;")?;
        let mut stmt = conn.prepare(
            "
            INSERT INTO todos (id, text, done, priority) VALUES (?1, ?2, ?3, ?4)
            ",
        )?;

        for todo in todos {
            let priority = format!("{:?}", todo.priority);
            stmt.execute(params![todo.id, todo.text, todo.done, priority])?;
        }
        Ok(())
    }
}

pub fn make_storage(config: &Config) -> anyhow::Result<Box<dyn Storage>> {
    match config.storage_backend {
        StorageBackend::Json => Ok(Box::new(FileStorage::new(config))),
        StorageBackend::Sqlite => Ok(Box::new(SqliteStorage::new(config)?)),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StorageBackend;
    use crate::models::Priority;
    use tempfile::tempdir;

    fn make_file_storage() -> (FileStorage, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let config = Config {
            storage_backend: StorageBackend::Json,
            storage_path: dir.path().join("todos.json"),
            database_path: dir.path().join("todos.db"),
        };
        (FileStorage::new(&config), dir)
    }

    fn make_sqlite_storage() -> (SqliteStorage, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let config = Config {
            storage_backend: StorageBackend::Sqlite,
            storage_path: dir.path().join("todos.json"),
            database_path: dir.path().join("todos.db"),
        };
        (SqliteStorage::new(&config).unwrap(), dir)
    }

    fn sample_todo(id: u32, text: &str, priority: Priority) -> Todo {
        Todo {
            id,
            text: text.to_string(),
            done: false,
            priority,
        }
    }

    #[test]
    fn test_file_load_returns_empty_when_no_file() {
        let (storage, _dir) = make_file_storage();
        let todos = storage.load().unwrap();
        assert!(todos.is_empty());
    }

    #[test]
    fn test_file_save_and_load_roundtrip() {
        let (storage, _dir) = make_file_storage();
        let todos = vec![sample_todo(1, "Buy milk", Priority::High)];
        storage.save(&todos).unwrap();
        let loaded = storage.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].text, "Buy milk");
        assert_eq!(loaded[0].priority, Priority::High);
    }

    #[test]
    fn test_file_save_overwrites_previous() {
        let (storage, _dir) = make_file_storage();
        storage
            .save(&[sample_todo(1, "Old", Priority::Low)])
            .unwrap();
        storage
            .save(&[sample_todo(1, "New", Priority::Low)])
            .unwrap();
        let loaded = storage.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].text, "New");
    }

    #[test]
    fn test_file_save_empty_list() {
        let (storage, _dir) = make_file_storage();
        storage
            .save(&[sample_todo(1, "Task", Priority::Low)])
            .unwrap();
        storage.save(&[]).unwrap();
        let loaded = storage.load().unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_sqlite_load_returns_empty_on_init() {
        let (storage, _dir) = make_sqlite_storage();
        let todos = storage.load().unwrap();
        assert!(todos.is_empty());
    }

    #[test]
    fn test_sqlite_save_and_load_roundtrip() {
        let (storage, _dir) = make_sqlite_storage();
        let todos = vec![sample_todo(1, "Buy milk", Priority::High)];
        storage.save(&todos).unwrap();
        let loaded = storage.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].text, "Buy milk");
        assert_eq!(loaded[0].priority, Priority::High);
    }

    #[test]
    fn test_sqlite_save_overwrites_previous() {
        let (storage, _dir) = make_sqlite_storage();
        storage
            .save(&[sample_todo(1, "Old", Priority::Low)])
            .unwrap();
        storage
            .save(&[sample_todo(1, "New", Priority::Low)])
            .unwrap();
        let loaded = storage.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].text, "New");
    }

    #[test]
    fn test_sqlite_save_empty_list() {
        let (storage, _dir) = make_sqlite_storage();
        storage
            .save(&[sample_todo(1, "Task", Priority::Low)])
            .unwrap();
        storage.save(&[]).unwrap();
        let loaded = storage.load().unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_sqlite_preserves_done_status() {
        let (storage, _dir) = make_sqlite_storage();
        let mut todo = sample_todo(1, "Task", Priority::Low);
        todo.done = true;
        storage.save(&[todo]).unwrap();
        let loaded = storage.load().unwrap();
        assert!(loaded[0].done);
    }

    #[test]
    fn test_sqlite_preserves_order() {
        let (storage, _dir) = make_sqlite_storage();
        let todos = vec![
            sample_todo(1, "First", Priority::Low),
            sample_todo(2, "Second", Priority::Medium),
            sample_todo(3, "Third", Priority::High),
        ];
        storage.save(&todos).unwrap();
        let loaded = storage.load().unwrap();
        assert_eq!(loaded[0].id, 1);
        assert_eq!(loaded[1].id, 2);
        assert_eq!(loaded[2].id, 3);
    }
}
