//! HTTP sync client

use crate::error::Result;

/// HTTP client for syncing with central API
pub struct HttpClient {
    // TODO: Implement HTTP client
}

impl HttpClient {
    /// Create new HTTP client
    pub fn new(base_url: &str) -> Self {
        Self {}
    }

    /// Push local changes to central
    pub fn push(&self) -> Result<()> {
        // TODO: Implement
        Ok(())
    }

    /// Pull changes from central
    pub fn pull(&self) -> Result<()> {
        // TODO: Implement
        Ok(())
    }
}

