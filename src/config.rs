use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    #[default]
    Json,
    Sqlite,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub storage_backend: StorageBackend,
    pub storage_path: PathBuf,
    pub database_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("todo_rs");
        Config {
            storage_backend: StorageBackend::default(),
            storage_path: dir.join("todos.json"),
            database_path: dir.join("todos.db"),
        }
    }
}

fn get_config_path() -> Result<PathBuf> {
    let mut path = dirs::config_dir().context("Could not find config directory")?;
    path.push("todo_rs");
    path.push("config.toml");
    Ok(path)
}

pub fn load_config() -> Result<Config> {
    let path = get_config_path()?;

    if !path.exists() {
        return Ok(Config::default());
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config: {:?}", path))?;

    let config = toml::from_str(&content).context("Config file is corrupted")?;

    Ok(config)
}

pub fn save_default_config() -> Result<()> {
    let path = get_config_path()?;

    if path.exists() {
        return Ok(());
    }

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create config dir: {:?}", dir))?;
    }

    let config = Config::default();
    let content = toml::to_string_pretty(&config).context("Failed to serialize config")?;

    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write config: {:?}", path))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config_backend_is_json() {
        let config = Config::default();
        assert_eq!(config.storage_backend, StorageBackend::Json);
    }

    #[test]
    fn test_default_config_paths_contain_todo_rs() {
        let config = Config::default();
        assert!(config.storage_path.to_str().unwrap().contains("todo_rs"));
        assert!(config.database_path.to_str().unwrap().contains("todo_rs"));
    }

    #[test]
    fn test_default_storage_path_is_json_file() {
        let config = Config::default();
        assert!(config.storage_path.to_str().unwrap().ends_with(".json"));
    }

    #[test]
    fn test_default_database_path_is_db_file() {
        let config = Config::default();
        assert!(config.database_path.to_str().unwrap().ends_with(".db"));
    }

    #[test]
    fn test_load_config_returns_default_when_missing() {
        let config = load_config().unwrap();
        assert!(config.storage_path.to_str().is_some());
        assert!(config.database_path.to_str().is_some());
    }

    #[test]
    fn test_config_roundtrip_json_backend() {
        let dir = tempdir().unwrap();
        let config = Config {
            storage_backend: StorageBackend::Json,
            storage_path: dir.path().join("todos.json"),
            database_path: dir.path().join("todos.db"),
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.storage_backend, StorageBackend::Json);
        assert_eq!(config.storage_path, parsed.storage_path);
        assert_eq!(config.database_path, parsed.database_path);
    }

    #[test]
    fn test_config_roundtrip_sqlite_backend() {
        let dir = tempdir().unwrap();
        let config = Config {
            storage_backend: StorageBackend::Sqlite,
            storage_path: dir.path().join("todos.json"),
            database_path: dir.path().join("todos.db"),
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.storage_backend, StorageBackend::Sqlite);
    }

    #[test]
    fn test_storage_backend_deserialize_json() {
        let toml_str = r#"
            storage_backend = "json"
            storage_path = "todos.json"
            database_path = "todos.db"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.storage_backend, StorageBackend::Json);
    }

    #[test]
    fn test_storage_backend_deserialize_sqlite() {
        let toml_str = r#"
            storage_backend = "sqlite"
            storage_path = "todos.json"
            database_path = "todos.db"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.storage_backend, StorageBackend::Sqlite);
    }
}
