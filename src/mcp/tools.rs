//! MCP tools for memory bank access

use crate::error::Result;

/// Query memory bank tool
pub fn query_memory_bank(query: &str) -> Result<String> {
    // TODO: Implement
    Ok(format!("Query: {}", query))
}

/// Store in memory bank tool
pub fn store_in_memory_bank(_context: &str) -> Result<()> {
    // TODO: Implement
    Ok(())
}

