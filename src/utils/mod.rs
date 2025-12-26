//! Utility functions

/// Get current timestamp
pub fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Format file path
pub fn format_path(path: &str) -> String {
    path.replace('\\', "/")
}

