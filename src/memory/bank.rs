//! Memory bank core implementation

use crate::error::Result;
use crate::memory::storage::StorageBackend;

/// Memory bank for storing and retrieving code patterns
pub struct MemoryBank {
    storage: Box<dyn StorageBackend>,
}

impl MemoryBank {
    /// Create a new memory bank
    pub fn new(storage: Box<dyn StorageBackend>) -> Self {
        Self { storage }
    }

    /// Store code context in memory bank
    pub fn store(&mut self, context: CodeContext) -> Result<()> {
        self.storage.store(context)
    }

    /// Delete entries with this exact `file_path` (for idempotent git captures).
    pub fn delete_by_file_path(&mut self, file_path: &str) -> Result<usize> {
        self.storage.delete_by_file_path(file_path)
    }

    /// Search for similar patterns
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<CodeContext>> {
        self.storage.search(query, limit)
    }

    /// Search with optional substring filter on stored file path
    pub fn search_filtered(
        &self,
        query: &str,
        file_substr: Option<&str>,
        limit: usize,
    ) -> Result<Vec<CodeContext>> {
        self.storage.search_filtered(query, file_substr, limit)
    }

    /// Get memory bank statistics
    pub fn stats(&self) -> Result<MemoryStats> {
        self.storage.stats()
    }

    /// Delete entries with timestamp strictly before `cutoff` (unix seconds).
    pub fn purge_before(&mut self, cutoff: u64) -> Result<usize> {
        self.storage.purge_before(cutoff)
    }

    /// Export newest entries up to `limit` (for backup / inspection).
    pub fn dump_entries(&self, limit: usize) -> Result<Vec<CodeContext>> {
        self.storage.dump_entries(limit)
    }
}

/// Code context stored in memory bank
#[derive(Debug, Clone)]
#[cfg_attr(feature = "storage-sqlite", derive(serde::Serialize))]
pub struct CodeContext {
    /// Code content
    pub content: String,
    
    /// File path
    pub file_path: String,
    
    /// Language
    pub language: String,
    
    /// Metadata
    pub metadata: Vec<(String, String)>,
    
    /// Timestamp
    pub timestamp: u64,
}

/// Memory bank statistics
#[derive(Debug)]
pub struct MemoryStats {
    /// Total entries
    pub total_entries: usize,
    
    /// Storage size in bytes
    pub storage_size: u64,
}

#[cfg(all(test, feature = "storage-sqlite"))]
mod tests {
    use super::*;
    use crate::memory::open_local_memory_bank;
    use tempfile::tempdir;

    #[test]
    fn sqlite_store_search_delete_roundtrip() {
        let dir = tempdir().unwrap();
        let mut bank = open_local_memory_bank(dir.path()).unwrap();
        let ctx = CodeContext {
            content: "alpha bravo unique_token".to_string(),
            file_path: "test/label".to_string(),
            language: "text".to_string(),
            metadata: vec![],
            timestamp: 1,
        };
        bank.store(ctx).unwrap();
        let hits = bank.search_filtered("unique_token", None, 5).unwrap();
        assert_eq!(hits.len(), 1);
        let n = bank.delete_by_file_path("test/label").unwrap();
        assert_eq!(n, 1);
        let hits2 = bank.search_filtered("unique_token", None, 5).unwrap();
        assert!(hits2.is_empty());
    }
}

