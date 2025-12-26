//! Integration tests for Joicy

use joicy::*;

#[test]
fn test_library_loads() {
    assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
}

#[test]
fn test_config_default() {
    let config = config::AppConfig::default();
    assert!(!config.memory.path.to_string_lossy().is_empty());
}

