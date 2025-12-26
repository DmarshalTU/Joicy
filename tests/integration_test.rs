//! Integration tests for Joicy

use joicy::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_library_loads() {
    assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
}

#[test]
fn test_config_default() {
    let config = config::AppConfig::default();
    assert!(!config.memory.path.to_string_lossy().is_empty());
    assert_eq!(config.memory.backend, "sqlite");
    assert_eq!(config.memory.vector_dim, 384);
}

#[test]
fn test_init_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_path = temp_dir.path().to_str().unwrap();
    
    // Test init command
    let result = joicy::cli::init(test_path);
    assert!(result.is_ok(), "Init command should succeed");
    
    // Verify directory structure was created
    let joicy_dir = PathBuf::from(test_path).join(".joicy");
    assert!(joicy_dir.exists(), ".joicy directory should exist");
    assert!(joicy_dir.is_dir(), ".joicy should be a directory");
    
    let memory_dir = joicy_dir.join("memory");
    assert!(memory_dir.exists(), "memory directory should exist");
    assert!(memory_dir.is_dir(), "memory should be a directory");
    
    let config_file = joicy_dir.join("config.toml");
    assert!(config_file.exists(), "config.toml should exist");
    assert!(config_file.is_file(), "config.toml should be a file");
    
    // Verify config file can be parsed
    let config_str = fs::read_to_string(&config_file).expect("Should read config file");
    let config: config::AppConfig = toml::from_str(&config_str)
        .expect("Config file should be valid TOML");
    assert_eq!(config.memory.backend, "sqlite");
}

#[test]
fn test_init_command_invalid_path() {
    // Test with non-existent path
    let result = joicy::cli::init("/nonexistent/path/that/does/not/exist");
    assert!(result.is_err(), "Init should fail with invalid path");
    
    // Test with file instead of directory
    let temp_file = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_file.path().join("test_file.txt");
    fs::write(&file_path, "test").expect("Should create test file");
    
    let result = joicy::cli::init(file_path.to_str().unwrap());
    assert!(result.is_err(), "Init should fail with file path");
}

