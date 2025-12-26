//! CLI module for Joicy

#[cfg(feature = "cli")]
mod commands;
#[cfg(feature = "cli")]
mod parser;

#[cfg(feature = "cli")]
pub use commands::*;
#[cfg(feature = "cli")]
pub use parser::*;

#[cfg(not(feature = "cli"))]
pub fn run() -> crate::error::Result<()> {
    Err(crate::error::Error::Config(
        "CLI feature is not enabled".to_string(),
    ))
}

