//! Memory bank module

mod bank;
mod storage;

pub use bank::*;
pub use storage::*;

use crate::error::Result;
#[cfg(not(feature = "storage-sqlite"))]
use crate::error::Error;
use std::path::Path;

/// Open the local memory bank for a repository using `.joicy/joicy.toml` (or legacy `config.toml`) settings.
#[cfg(feature = "storage-sqlite")]
pub fn open_local_memory_bank(memory_dir: &Path) -> Result<MemoryBank> {
    let storage = storage::sqlite::SqliteStorage::open(memory_dir)?;
    Ok(MemoryBank::new(Box::new(storage)))
}

/// When SQLite storage is disabled at compile time.
#[cfg(not(feature = "storage-sqlite"))]
pub fn open_local_memory_bank(_memory_dir: &Path) -> Result<MemoryBank> {
    Err(Error::Config(
        "Joicy was built without `storage-sqlite`. Rebuild with --features storage-sqlite."
            .to_string(),
    ))
}

