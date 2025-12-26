//! Error types for Joicy

use thiserror::Error;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for Joicy
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Memory bank errors
    #[error("Memory bank error: {0}")]
    Memory(String),

    /// Git integration errors
    #[error("Git error: {0}")]
    Git(String),

    /// MCP server errors
    #[error("MCP error: {0}")]
    Mcp(String),

    /// Sync errors
    #[error("Sync error: {0}")]
    Sync(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Network errors
    #[error("Network error: {0}")]
    Network(String),

    /// Storage errors
    #[error("Storage error: {0}")]
    Storage(String),
}

