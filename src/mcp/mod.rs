//! Model Context Protocol (stdio server for IDEs).

#[cfg(all(feature = "mcp", feature = "storage-sqlite"))]
mod serve;

#[cfg(all(feature = "mcp", feature = "storage-sqlite"))]
pub use serve::serve_stdio;

/// When MCP or SQLite is disabled at compile time.
#[cfg(not(all(feature = "mcp", feature = "storage-sqlite")))]
pub fn serve_stdio() -> crate::error::Result<()> {
    Err(crate::error::Error::Mcp(
        "`joicy mcp serve` requires `mcp` and `storage-sqlite` (enabled in default features).".into(),
    ))
}
