use crate::models::{Priority, Todo};
use crate::storage::Storage;
use anyhow::Result;

pub fn add_todo(storage: &dyn Storage, text: String, priority: Priority) -> Result<Vec<Todo>> {
    let mut todos = storage.load()?;
    let id = todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    todos.push(Todo {
        id,
        text,
        done: false,
        priority,
    });
    storage.save(&todos)?;
    Ok(todos)
}

pub fn list_todos(storage: &dyn Storage, filter: Option<&str>) -> Result<Vec<Todo>> {
    let todos = storage.load()?;
    let filtered = match filter {
        Some("done") => todos.into_iter().filter(|t| t.done).collect(),
        Some("pending") => todos.into_iter().filter(|t| !t.done).collect(),
        _ => todos,
    };
    Ok(filtered)
}

fn set_done(storage: &dyn Storage, id: u32, done: bool) -> Result<Option<Vec<Todo>>> {
    let mut todos = storage.load()?;
    let Some(todo) = todos.iter_mut().find(|t| t.id == id) else {
        return Ok(None);
    };
    todo.done = done;
    storage.save(&todos)?;
    Ok(Some(todos))
}

pub fn mark_done(storage: &dyn Storage, id: u32) -> Result<Option<Vec<Todo>>> {
    set_done(storage, id, true)
}

pub fn mark_undone(storage: &dyn Storage, id: u32) -> Result<Option<Vec<Todo>>> {
    set_done(storage, id, false)
}

pub fn remove_todo(storage: &dyn Storage, id: u32) -> Result<Option<Vec<Todo>>> {
    let mut todos = storage.load()?;
    let len_before = todos.len();
    todos.retain(|t| t.id != id);
    if todos.len() == len_before {
        return Ok(None);
    }
    storage.save(&todos)?;
    Ok(Some(todos))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::TodoError;
    use crate::models::{Priority, Todo};
    use std::cell::RefCell;

    struct FakeStorage {
        todos: RefCell<Vec<Todo>>,
    }

    impl FakeStorage {
        fn empty() -> Self {
            FakeStorage {
                todos: RefCell::new(vec![]),
            }
        }
    }

    impl Storage for FakeStorage {
        fn load(&self) -> Result<Vec<Todo>, TodoError> {
            Ok(self.todos.borrow().clone())
        }
        fn save(&self, todos: &[Todo]) -> Result<(), TodoError> {
            *self.todos.borrow_mut() = todos.to_vec();
            Ok(())
        }
    }

    #[test]
    fn test_add_todo() {
        let storage = FakeStorage::empty();
        let todos = add_todo(&storage, "Buy milk".to_string(), Priority::Medium).unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].text, "Buy milk");
        assert_eq!(todos[0].id, 1);
        assert_eq!(todos[0].priority, Priority::Medium);
    }

    #[test]
    fn test_add_todo_with_priority() {
        let storage = FakeStorage::empty();
        let todos = add_todo(&storage, "Important".to_string(), Priority::High).unwrap();
        assert_eq!(todos[0].priority, Priority::High);
    }

    #[test]
    fn test_remove_nonexistent_returns_none() {
        let storage = FakeStorage::empty();
        let result = remove_todo(&storage, 99).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_mark_done() {
        let storage = FakeStorage::empty();
        add_todo(&storage, "Task".to_string(), Priority::Low).unwrap();
        let todos = mark_done(&storage, 1).unwrap().unwrap();
        assert!(todos[0].done);
    }
    #[test]
    fn test_mark_undone() {
        let storage = FakeStorage::empty();
        add_todo(&storage, "Task".to_string(), Priority::Low).unwrap();
        mark_done(&storage, 1).unwrap();
        let todos = mark_undone(&storage, 1).unwrap().unwrap();
        assert!(!todos[0].done);
    }

    #[test]
    fn test_mark_undone_nonexistent_returns_none() {
        let storage = FakeStorage::empty();
        let result = mark_undone(&storage, 99).unwrap();
        assert!(result.is_none());
    }
    #[test]
    fn test_add_multiple_ids_increment() {
        let storage = FakeStorage::empty();
        add_todo(&storage, "First".to_string(), Priority::Low).unwrap();
        add_todo(&storage, "Second".to_string(), Priority::Low).unwrap();
        let todos = add_todo(&storage, "Third".to_string(), Priority::Low).unwrap();
        assert_eq!(todos[0].id, 1);
        assert_eq!(todos[1].id, 2);
        assert_eq!(todos[2].id, 3);
    }

    #[test]
    fn test_list_filter_done() {
        let storage = FakeStorage::empty();
        add_todo(&storage, "First".to_string(), Priority::Low).unwrap();
        add_todo(&storage, "Second".to_string(), Priority::Low).unwrap();
        mark_done(&storage, 1).unwrap();
        let todos = list_todos(&storage, Some("done")).unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].id, 1);
    }

    #[test]
    fn test_list_filter_pending() {
        let storage = FakeStorage::empty();
        add_todo(&storage, "First".to_string(), Priority::Low).unwrap();
        add_todo(&storage, "Second".to_string(), Priority::Low).unwrap();
        mark_done(&storage, 1).unwrap();
        let todos = list_todos(&storage, Some("pending")).unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].id, 2);
    }

    #[test]
    fn test_list_no_filter_returns_all() {
        let storage = FakeStorage::empty();
        add_todo(&storage, "First".to_string(), Priority::Low).unwrap();
        add_todo(&storage, "Second".to_string(), Priority::Low).unwrap();
        let todos = list_todos(&storage, None).unwrap();
        assert_eq!(todos.len(), 2);
    }

    #[test]
    fn test_mark_done_nonexistent_returns_none() {
        let storage = FakeStorage::empty();
        let result = mark_done(&storage, 99).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_remove_todo() {
        let storage = FakeStorage::empty();
        add_todo(&storage, "Task".to_string(), Priority::Low).unwrap();
        let result = remove_todo(&storage, 1).unwrap();
        assert!(result.is_some());
        let todos = list_todos(&storage, None).unwrap();
        assert!(todos.is_empty());
    }
}
