//! Sync service module

#[cfg(feature = "sync-http")]
mod http;
#[cfg(feature = "sync-http")]
mod service;

#[cfg(feature = "sync-http")]
pub use service::*;

/// Sync with central memory bank
#[cfg(not(feature = "sync-http"))]
pub fn sync_with_central() -> crate::error::Result<()> {
    Err(crate::error::Error::Sync(
        "Sync feature is not enabled".to_string(),
    ))
}

