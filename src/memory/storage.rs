//! Storage backend trait and implementations

use crate::error::Result;
use crate::memory::bank::{CodeContext, MemoryStats};

/// Trait for storage backends
pub trait StorageBackend: Send + Sync {
    /// Store code context
    fn store(&mut self, context: CodeContext) -> Result<()>;
    
    /// Search for similar patterns
    fn search(&self, query: &str, limit: usize) -> Result<Vec<CodeContext>>;
    
    /// Get statistics
    fn stats(&self) -> Result<MemoryStats>;
}

/// SQLite storage backend
#[cfg(feature = "storage-sqlite")]
pub mod sqlite {
    use super::*;
    use crate::error::Error;
    
    pub struct SqliteStorage {
        // TODO: Implement SQLite storage
    }
    
    impl StorageBackend for SqliteStorage {
        fn store(&mut self, _context: CodeContext) -> Result<()> {
            Err(Error::Storage("Not implemented".to_string()))
        }
        
        fn search(&self, _query: &str, _limit: usize) -> Result<Vec<CodeContext>> {
            Err(Error::Storage("Not implemented".to_string()))
        }
        
        fn stats(&self) -> Result<MemoryStats> {
            Err(Error::Storage("Not implemented".to_string()))
        }
    }
}

/// Qdrant storage backend
#[cfg(feature = "storage-qdrant")]
pub mod qdrant {
    use super::*;
    use crate::error::Error;
    
    pub struct QdrantStorage {
        // TODO: Implement Qdrant storage
    }
    
    impl StorageBackend for QdrantStorage {
        fn store(&mut self, _context: CodeContext) -> Result<()> {
            Err(Error::Storage("Not implemented".to_string()))
        }
        
        fn search(&self, _query: &str, _limit: usize) -> Result<Vec<CodeContext>> {
            Err(Error::Storage("Not implemented".to_string()))
        }
        
        fn stats(&self) -> Result<MemoryStats> {
            Err(Error::Storage("Not implemented".to_string()))
        }
    }
}

