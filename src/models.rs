use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum Priority {
    #[default]
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u32,
    pub text: String,
    pub done: bool,
    #[serde(default)]
    pub priority: Priority,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_priority_is_low() {
        assert_eq!(Priority::default(), Priority::Low);
    }

    #[test]
    fn test_deserialize_todo_without_priority() {
        let json = r#"{"id":1,"text":"Task","done":false}"#;
        let todo: Todo = serde_json::from_str(json).unwrap();
        assert_eq!(todo.priority, Priority::Low);
    }

    #[test]
    fn test_deserialize_todo_with_priority() {
        let json = r#"{"id":1,"text":"Task","done":false,"priority":"High"}"#;
        let todo: Todo = serde_json::from_str(json).unwrap();
        assert_eq!(todo.priority, Priority::High);
    }
}
