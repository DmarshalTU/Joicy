//! Joicy - Team Memory Bank System
//!
//! A system for capturing, storing, and sharing developer knowledge across teams.
//! Enables AI agents to learn from team history and provide context-aware assistance.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod cli;
pub mod config;
pub mod error;
pub mod git;
pub mod memory;
pub mod mcp;
pub mod sync;
pub mod utils;

// Re-export commonly used types
pub use error::{Error, Result};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

