//! Storage backend trait and implementations

use crate::error::Result;
use crate::memory::bank::{CodeContext, MemoryStats};

/// Trait for storage backends
pub trait StorageBackend: Send + Sync {
    /// Store code context
    fn store(&mut self, context: CodeContext) -> Result<()>;

    /// Search for similar patterns
    fn search(&self, query: &str, limit: usize) -> Result<Vec<CodeContext>>;

    /// Search with optional file path substring filter
    fn search_filtered(
        &self,
        query: &str,
        file_substr: Option<&str>,
        limit: usize,
    ) -> Result<Vec<CodeContext>> {
        let mut hits = self.search(query, limit)?;
        if let Some(sub) = file_substr {
            hits.retain(|c| c.file_path.contains(sub));
        }
        Ok(hits)
    }

    /// Get statistics
    fn stats(&self) -> Result<MemoryStats>;

    /// Delete rows whose logical `file_path` matches exactly (e.g. replace a git capture).
    fn delete_by_file_path(&mut self, file_path: &str) -> Result<usize> {
        let _ = file_path;
        Ok(0)
    }

    /// Remove entries older than `cutoff` (unix seconds). Default: unsupported.
    fn purge_before(&mut self, _cutoff: u64) -> Result<usize> {
        Err(crate::error::Error::Storage(
            "purge not supported for this backend".to_string(),
        ))
    }

    /// Dump entries for export (newest first). Default: unsupported.
    fn dump_entries(&self, _limit: usize) -> Result<Vec<CodeContext>> {
        Err(crate::error::Error::Storage(
            "export not supported for this backend".to_string(),
        ))
    }
}

/// SQLite + FTS5 storage (full-text search; POC local dev — no embeddings yet).
#[cfg(feature = "storage-sqlite")]
pub mod sqlite {
    use super::*;
    use crate::error::Error;
    use rusqlite::{params, Connection};
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    const SCHEMA: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
    content,
    file_path,
    language,
    metadata,
    timestamp_text,
    tokenize = 'porter unicode61'
);
"#;

    /// Local SQLite memory bank
    pub struct SqliteStorage {
        conn: Mutex<Connection>,
        db_path: PathBuf,
    }

    impl SqliteStorage {
        /// Open or create storage at `memory_dir/joicy.db`.
        pub fn open(memory_dir: &Path) -> Result<Self> {
            std::fs::create_dir_all(memory_dir).map_err(|e| {
                Error::Storage(format!("Failed to create memory directory: {e}"))
            })?;
            let db_path = memory_dir.join("joicy.db");
            let conn = Connection::open(&db_path)
                .map_err(|e| Error::Storage(format!("Failed to open database: {e}")))?;
            conn.execute_batch(SCHEMA)
                .map_err(|e| Error::Storage(format!("Failed to init schema: {e}")))?;
            Ok(Self {
                conn: Mutex::new(conn),
                db_path,
            })
        }

        fn row_to_context(
            content: String,
            file_path: String,
            language: String,
            metadata: String,
            timestamp_text: String,
        ) -> Result<CodeContext> {
            let timestamp: u64 = timestamp_text
                .parse()
                .map_err(|_| Error::Storage("invalid timestamp in row".to_string()))?;
            let metadata: Vec<(String, String)> = if metadata.is_empty() {
                Vec::new()
            } else {
                serde_json::from_str(&metadata).map_err(|e| {
                    Error::Storage(format!("metadata decode error: {e}"))
                })?
            };
            Ok(CodeContext {
                content,
                file_path,
                language,
                metadata,
                timestamp,
            })
        }
    }

    fn metadata_json(meta: &[(String, String)]) -> Result<String> {
        serde_json::to_string(meta).map_err(|e| {
            Error::Serialization(format!("metadata encode error: {e}"))
        })
    }

    /// Build a conservative FTS5 `MATCH` query (tokens ANDed, quoted).
    pub fn fts_match_query(raw: &str) -> String {
        let tokens: Vec<&str> = raw.split_whitespace().filter(|t| !t.is_empty()).collect();
        if tokens.is_empty() {
            return String::new();
        }
        tokens
            .iter()
            .map(|t| {
                let escaped = t.replace('"', "\"\"");
                format!("\"{escaped}\"")
            })
            .collect::<Vec<_>>()
            .join(" AND ")
    }

    impl StorageBackend for SqliteStorage {
        fn store(&mut self, context: CodeContext) -> Result<()> {
            let meta = metadata_json(&context.metadata)?;
            let ts = context.timestamp.to_string();
            let conn = self.conn.lock().map_err(|e| {
                Error::Storage(format!("database lock error: {e}"))
            })?;
            conn.execute(
                "INSERT INTO memory_fts (content, file_path, language, metadata, timestamp_text) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    context.content,
                    context.file_path,
                    context.language,
                    meta,
                    ts,
                ],
            )
            .map_err(|e| Error::Storage(format!("insert failed: {e}")))?;
            Ok(())
        }

        fn delete_by_file_path(&mut self, file_path: &str) -> Result<usize> {
            let conn = self.conn.lock().map_err(|e| {
                Error::Storage(format!("database lock error: {e}"))
            })?;
            let n = conn
                .execute(
                    "DELETE FROM memory_fts WHERE file_path = ?1",
                    params![file_path],
                )
                .map_err(|e| Error::Storage(format!("delete_by_file_path failed: {e}")))?;
            Ok(n)
        }

        fn search(&self, query: &str, limit: usize) -> Result<Vec<CodeContext>> {
            self.search_filtered(query, None, limit)
        }

        fn search_filtered(
            &self,
            query: &str,
            file_substr: Option<&str>,
            limit: usize,
        ) -> Result<Vec<CodeContext>> {
            let conn = self.conn.lock().map_err(|e| {
                Error::Storage(format!("database lock error: {e}"))
            })?;
            let lim = i64::try_from(limit).unwrap_or(100);
            let q = query.trim();
            let mut out = Vec::new();

            if q.is_empty() {
                let mut stmt = conn
                    .prepare(
                        "SELECT content, file_path, language, metadata, timestamp_text FROM memory_fts ORDER BY rowid DESC LIMIT ?1",
                    )
                    .map_err(|e| Error::Storage(format!("prepare failed: {e}")))?;
                let rows = stmt.query_map(params![lim], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                });
                let rows = rows.map_err(|e| Error::Storage(format!("query failed: {e}")))?;
                for r in rows {
                    let (c, fp, lang, meta, ts) =
                        r.map_err(|e| Error::Storage(format!("row error: {e}")))?;
                    let ctx = Self::row_to_context(c, fp, lang, meta, ts)?;
                    if let Some(sub) = file_substr
                        && !ctx.file_path.contains(sub)
                    {
                        continue;
                    }
                    out.push(ctx);
                }
                return Ok(out);
            }

            let fts_q = fts_match_query(q);
            if fts_q.is_empty() {
                return Ok(out);
            }

            let sql = match file_substr {
                Some(_) => {
                    "SELECT content, file_path, language, metadata, timestamp_text FROM memory_fts WHERE memory_fts MATCH ?1 ORDER BY rank LIMIT ?2"
                }
                None => {
                    "SELECT content, file_path, language, metadata, timestamp_text FROM memory_fts WHERE memory_fts MATCH ?1 ORDER BY rank LIMIT ?2"
                }
            };

            let mut stmt = conn
                .prepare(sql)
                .map_err(|e| Error::Storage(format!("prepare failed: {e}")))?;
            let rows = stmt.query_map(params![fts_q, lim], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            });
            let rows = rows.map_err(|e| Error::Storage(format!("query failed: {e}")))?;
            for r in rows {
                let (c, fp, lang, meta, ts) =
                    r.map_err(|e| Error::Storage(format!("row error: {e}")))?;
                let ctx = Self::row_to_context(c, fp, lang, meta, ts)?;
                if let Some(sub) = file_substr
                    && !ctx.file_path.contains(sub)
                {
                    continue;
                }
                out.push(ctx);
            }
            Ok(out)
        }

        fn stats(&self) -> Result<MemoryStats> {
            let conn = self.conn.lock().map_err(|e| {
                Error::Storage(format!("database lock error: {e}"))
            })?;
            let total: i64 = conn
                .query_row("SELECT count(*) FROM memory_fts", [], |row| row.get(0))
                .map_err(|e| Error::Storage(format!("stats query failed: {e}")))?;
            let storage_size = std::fs::metadata(&self.db_path)
                .map(|m| m.len())
                .unwrap_or(0);
            Ok(MemoryStats {
                total_entries: total as usize,
                storage_size,
            })
        }

        fn purge_before(&mut self, cutoff: u64) -> Result<usize> {
            let conn = self.conn.lock().map_err(|e| {
                Error::Storage(format!("database lock error: {e}"))
            })?;
            let n = conn
                .execute(
                    "DELETE FROM memory_fts WHERE CAST(timestamp_text AS INTEGER) < ?1",
                    params![cutoff.to_string()],
                )
                .map_err(|e| Error::Storage(format!("purge failed: {e}")))?;
            Ok(n)
        }

        fn dump_entries(&self, limit: usize) -> Result<Vec<CodeContext>> {
            let conn = self.conn.lock().map_err(|e| {
                Error::Storage(format!("database lock error: {e}"))
            })?;
            let lim = i64::try_from(limit).unwrap_or(100_000);
            let mut stmt = conn
                .prepare(
                    "SELECT content, file_path, language, metadata, timestamp_text FROM memory_fts ORDER BY rowid DESC LIMIT ?1",
                )
                .map_err(|e| Error::Storage(format!("prepare failed: {e}")))?;
            let rows = stmt.query_map(params![lim], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            });
            let rows = rows.map_err(|e| Error::Storage(format!("query failed: {e}")))?;
            let mut out = Vec::new();
            for r in rows {
                let (c, fp, lang, meta, ts) =
                    r.map_err(|e| Error::Storage(format!("row error: {e}")))?;
                out.push(Self::row_to_context(c, fp, lang, meta, ts)?);
            }
            Ok(out)
        }
    }
}

/// Qdrant storage backend
#[cfg(feature = "storage-qdrant")]
pub mod qdrant {
    use super::*;
    use crate::error::Error;

    pub struct QdrantStorage {
        // Stub for optional `storage-qdrant`; not wired in this revision.
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
