//! MCP (Model Context Protocol) server module

#[cfg(feature = "mcp")]
mod server;
#[cfg(feature = "mcp")]
mod tools;

#[cfg(feature = "mcp")]
pub use server::*;
#[cfg(feature = "mcp")]
pub use tools::*;

#[cfg(not(feature = "mcp"))]
pub fn start_server() -> crate::error::Result<()> {
    Err(crate::error::Error::Mcp(
        "MCP feature is not enabled".to_string(),
    ))
}

