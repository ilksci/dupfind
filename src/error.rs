use thiserror::Error;

/// Unified error type for all dupfind modules.
#[derive(Error, Debug)]
pub enum DupfindError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Directory traversal error: {0}")]
    Walkdir(#[from] walkdir::Error),

    #[error("CSV report error: {0}")]
    Csv(#[from] csv::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid size format: '{0}'. Use numbers with optional suffix: B, KB, MB, GB (e.g. 1MB, 500KB)")]
    InvalidSize(String),

    #[error("{0}")]
    Other(String),
}

/// Convenience type alias used across the crate.
pub type Result<T> = std::result::Result<T, DupfindError>;
