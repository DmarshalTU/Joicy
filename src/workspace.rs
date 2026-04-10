//! Resolve Joicy repository root and open the local memory bank (shared by CLI and MCP).

use crate::config::AppConfig;
use crate::error::{Error, Result};
use crate::memory::{open_local_memory_bank, MemoryBank};
use std::fs;
use std::path::{Path, PathBuf};

/// Primary config filename under `.joicy/` (avoids clashing with unrelated `config.toml` in the repo).
pub const JOICY_CONFIG_TOML: &str = "joicy.toml";

/// Legacy config filename (still read if `joicy.toml` is absent).
pub const JOICY_CONFIG_LEGACY: &str = "config.toml";

fn joicy_dir_has_config(joicy_dir: &Path) -> bool {
    joicy_dir.join(JOICY_CONFIG_TOML).is_file()
        || joicy_dir.join(JOICY_CONFIG_LEGACY).is_file()
}

/// Walk up from `start` until `.joicy/joicy.toml` or legacy `.joicy/config.toml` is found.
pub fn find_joicy_root_from(start: &Path) -> Result<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        let joicy_dir = dir.join(".joicy");
        if joicy_dir_has_config(&joicy_dir) {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err(Error::Config(
        "No Joicy config found (.joicy/joicy.toml or .joicy/config.toml). Run `joicy init` in your repository root."
            .to_string(),
    ))
}

/// Like [`find_joicy_root_from`] but starts from [`std::env::current_dir`].
pub fn find_joicy_root() -> Result<PathBuf> {
    let dir = std::env::current_dir().map_err(|e| {
        Error::Config(format!("Cannot get current directory: {e}"))
    })?;
    find_joicy_root_from(&dir)
}

/// Prefer `JOICY_REPO_ROOT` when set and valid; otherwise discover from cwd.
pub fn resolve_repo_root() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("JOICY_REPO_ROOT") {
        let pb = PathBuf::from(p.trim());
        let jd = pb.join(".joicy");
        if joicy_dir_has_config(&jd) {
            return Ok(pb);
        }
        return Err(Error::Config(format!(
            "JOICY_REPO_ROOT is set to {} but .joicy/{} (or legacy config.toml) is missing there",
            pb.display(),
            JOICY_CONFIG_TOML
        )));
    }
    find_joicy_root()
}

/// Resolve which config file to load: prefers `.joicy/joicy.toml`, then legacy `config.toml`.
pub fn resolve_repo_config_path(repo_root: &Path) -> Result<PathBuf> {
    let joicy = repo_root.join(".joicy");
    let primary = joicy.join(JOICY_CONFIG_TOML);
    let legacy = joicy.join(JOICY_CONFIG_LEGACY);
    if primary.is_file() {
        Ok(primary)
    } else if legacy.is_file() {
        Ok(legacy)
    } else {
        Err(Error::Config(format!(
            "No config at {} or {}",
            primary.display(),
            legacy.display()
        )))
    }
}

/// Load Joicy repo config (see [`resolve_repo_config_path`]).
pub fn load_repo_config(repo_root: &Path) -> Result<AppConfig> {
    let config_file = resolve_repo_config_path(repo_root)?;
    let config_str = fs::read_to_string(&config_file).map_err(|e| {
        Error::Config(format!("Failed to read {}: {e}", config_file.display()))
    })?;
    toml::from_str(&config_str).map_err(|e| {
        Error::Config(format!("Invalid configuration file format: {e}"))
    })
}

/// Resolve configured memory directory to an absolute path.
pub fn resolve_memory_dir(repo_root: &Path, cfg: &AppConfig) -> PathBuf {
    if cfg.memory.path.is_absolute() {
        cfg.memory.path.clone()
    } else {
        repo_root.join(&cfg.memory.path)
    }
}

/// Resolve Obsidian vault root and namespace folder name (shared by CLI export and automation).
pub fn resolve_vault_paths(
    repo_root: &Path,
    cfg: &AppConfig,
    dir_override: Option<&std::path::Path>,
    ns_override: Option<&str>,
) -> (PathBuf, String) {
    let ns = ns_override
        .map(String::from)
        .or_else(|| cfg.vault.namespace.clone())
        .unwrap_or_else(|| {
            repo_root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("repo")
                .to_string()
        });
    let root = dir_override
        .map(PathBuf::from)
        .or_else(|| cfg.vault.export_root.clone())
        .or_else(|| std::env::var_os("JOICY_VAULT_ROOT").map(PathBuf::from))
        .unwrap_or_else(|| repo_root.join(".joicy").join("vault"));
    (root, ns)
}

/// Open the [`MemoryBank`] for this repo (SQLite when `memory.backend = "sqlite"`).
pub fn open_bank(repo_root: &Path, cfg: &AppConfig) -> Result<MemoryBank> {
    let dir = resolve_memory_dir(repo_root, cfg);
    match cfg.memory.backend.as_str() {
        "sqlite" => open_local_memory_bank(&dir),
        other => Err(Error::Config(format!(
            "Unsupported memory backend '{other}'. For local POC set memory.backend = \"sqlite\"."
        ))),
    }
}
