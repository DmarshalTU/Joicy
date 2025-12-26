//! Configuration management

use crate::error::{Error, Result};
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Memory bank configuration
    pub memory: MemoryConfig,
    
    /// Git integration configuration
    #[cfg(feature = "git")]
    pub git: GitConfig,
    
    /// MCP server configuration
    #[cfg(feature = "mcp")]
    pub mcp: McpConfig,
    
    /// Sync configuration
    #[cfg(feature = "sync-http")]
    pub sync: SyncConfig,
}

/// Memory bank configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Storage backend type
    pub backend: String,
    
    /// Storage path
    pub path: PathBuf,
    
    /// Vector dimension
    pub vector_dim: usize,
    
    /// Cache configuration
    #[cfg(feature = "cache-memory")]
    pub cache: CacheConfig,
}

/// Git configuration
#[cfg(feature = "git")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// Enable git hooks
    pub enable_hooks: bool,
    
    /// Hook installation path
    pub hooks_path: PathBuf,
}

/// MCP server configuration
#[cfg(feature = "mcp")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Enable MCP server
    pub enabled: bool,
    
    /// MCP server port
    pub port: u16,
}

/// Sync configuration
#[cfg(feature = "sync-http")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Central API endpoint
    pub central_url: Option<String>,
    
    /// Sync interval in seconds
    pub sync_interval: u64,
    
    /// API key for authentication
    pub api_key: Option<String>,
}

/// Cache configuration
#[cfg(feature = "cache-memory")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache size limit
    pub size_limit: usize,
    
    /// Cache TTL in seconds
    pub ttl: u64,
}

impl AppConfig {
    /// Load configuration from file and environment
    pub fn load() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| Error::Config("Cannot find config directory".to_string()))?
            .join("joicy");

        let config = Config::builder()
            .add_source(File::with_name(
                config_dir.join("config").to_str().unwrap_or("config"),
            ).required(false))
            .add_source(Environment::with_prefix("JOICY"))
            .build()
            .map_err(|e| Error::Config(e.to_string()))?;

        config
            .try_deserialize()
            .map_err(|e| Error::Config(e.to_string()))
    }

    /// Get default configuration
    pub fn default() -> Self {
        Self {
            memory: MemoryConfig {
                backend: "sqlite".to_string(),
                path: dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("joicy")
                    .join("memory"),
                vector_dim: 384,
                #[cfg(feature = "cache-memory")]
                cache: CacheConfig {
                    size_limit: 1000,
                    ttl: 3600,
                },
            },
            #[cfg(feature = "git")]
            git: GitConfig {
                enable_hooks: true,
                hooks_path: PathBuf::from(".git/hooks"),
            },
            #[cfg(feature = "mcp")]
            mcp: McpConfig {
                enabled: true,
                port: 8080,
            },
            #[cfg(feature = "sync-http")]
            sync: SyncConfig {
                central_url: None,
                sync_interval: 300,
                api_key: None,
            },
        }
    }
}

