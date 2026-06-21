use thiserror::Error;

#[derive(Debug, Error)]
pub enum TodoError {
    #[error("Failed to read storage: {0}")]
    Io(#[from] std::io::Error),

    #[error("Storage file is corrupted: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),
}
