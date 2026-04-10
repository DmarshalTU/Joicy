//! Configuration management

use crate::error::{Error, Result};
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Markdown vault / Obsidian export settings (dimension 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    /// Shared vault root for Obsidian (same path in every repo = one vault with namespaces).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub export_root: Option<PathBuf>,
    /// Subfolder name under `export_root` (default: repository directory name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// After each automated capture, refresh markdown files under the vault root.
    #[serde(default = "default_vault_auto_export")]
    pub auto_export: bool,
}

fn default_vault_auto_export() -> bool {
    true
}

/// Local post-commit pipeline (SQLite + vault + changelog + ticket notes).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    /// Append a line to the repo changelog on each commit (step 3 of 4).
    #[serde(default = "default_automation_changelog")]
    pub changelog_on_commit: bool,
    /// Changelog file relative to repo root (default `CHANGELOG.md`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changelog_path: Option<PathBuf>,
    /// Write a short Obsidian-friendly stub under the vault namespace (step 4 of 4).
    #[serde(default = "default_automation_ticket")]
    pub ticket_note_on_commit: bool,
    /// Subfolder under the vault namespace for ticket stubs, e.g. `tickets/`.
    #[serde(default = "default_ticket_vault_subdir")]
    pub ticket_vault_subdir: String,
}

fn default_automation_changelog() -> bool {
    true
}

fn default_automation_ticket() -> bool {
    true
}

fn default_ticket_vault_subdir() -> String {
    "tickets".to_string()
}

impl Default for AutomationConfig {
    fn default() -> Self {
        Self {
            changelog_on_commit: default_automation_changelog(),
            changelog_path: None,
            ticket_note_on_commit: default_automation_ticket(),
            ticket_vault_subdir: default_ticket_vault_subdir(),
        }
    }
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            export_root: None,
            namespace: None,
            auto_export: true,
        }
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Memory bank configuration
    pub memory: MemoryConfig,

    /// Vault export defaults (Obsidian). Can be overridden with `JOICY_VAULT_ROOT`.
    #[serde(default)]
    pub vault: VaultConfig,

    /// Post-commit automation (changelog + ticket stubs alongside memory + vault).
    #[serde(default)]
    pub automation: AutomationConfig,

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

}

impl Default for AppConfig {
    /// Get default configuration
    fn default() -> Self {
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
            vault: VaultConfig::default(),
            automation: AutomationConfig::default(),
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

