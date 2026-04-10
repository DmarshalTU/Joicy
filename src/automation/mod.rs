//! Changelog file and post-commit extras (changelog line + Obsidian ticket stubs).
//!
//! Together with SQLite capture and vault export, this completes the **four-way local trigger**
//! on each `git commit` (when `joicy hooks install` is used).

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::AppConfig;
use crate::error::Result;

/// Resolved path to the repo changelog file.
pub fn changelog_file_path(repo_root: &Path, cfg: &AppConfig) -> PathBuf {
    let rel = cfg
        .automation
        .changelog_path
        .as_deref()
        .unwrap_or_else(|| Path::new("CHANGELOG.md"));
    repo_root.join(rel)
}

/// Last `max_lines` lines of the changelog (for CLI / MCP).
pub fn read_changelog_tail(repo_root: &Path, cfg: &AppConfig, max_lines: usize) -> Result<String> {
    let path = changelog_file_path(repo_root, cfg);
    if !path.is_file() {
        return Ok(format!("(no changelog file at {})", path.display()));
    }
    let text = fs::read_to_string(&path)?;
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return Ok(String::new());
    }
    let take = max_lines.min(lines.len());
    let start = lines.len() - take;
    Ok(lines[start..].join("\n"))
}

/// Append one commit line to the changelog. Skips if this commit hash is already recorded.
#[cfg(feature = "git")]
pub fn append_changelog_entry(
    repo_root: &Path,
    cfg: &AppConfig,
    meta: &crate::git::CommitMeta,
) -> Result<()> {
    if !cfg.automation.changelog_on_commit {
        return Ok(());
    }
    let path = changelog_file_path(repo_root, cfg);
    let day = meta
        .date_iso
        .split('T')
        .next()
        .unwrap_or(meta.date_iso.as_str())
        .to_string();
    let marker = format!("<!-- joicy:commit:{} -->", meta.full_hash);
    let line = format!(
        "- {} — `{}` — {} {}\n",
        day, meta.short, meta.subject, marker
    );

    if path.is_file() {
        let existing = fs::read_to_string(&path)?;
        if existing.contains(&marker) {
            return Ok(());
        }
        let mut out = existing;
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&line);
        fs::write(&path, out)?;
    } else {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let header = "# Changelog\n\n\
             All notable changes to this project are recorded here. \
             New lines are appended automatically by Joicy on each `git commit` (local POC).\n\n";
        fs::write(&path, format!("{header}{line}"))?;
    }
    Ok(())
}

#[cfg(feature = "git")]
fn yaml_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\n', '\r'], " ")
}

/// Per-commit stub under `{vault}/{namespace}/{ticket_vault_subdir}/` for Obsidian.
#[cfg(feature = "git")]
pub fn write_ticket_stub(
    repo_root: &Path,
    cfg: &AppConfig,
    meta: &crate::git::CommitMeta,
) -> Result<()> {
    if !cfg.automation.ticket_note_on_commit || !cfg.vault.auto_export {
        return Ok(());
    }
    let (vault_root, ns) = crate::workspace::resolve_vault_paths(repo_root, cfg, None, None);
    let sub = cfg.automation.ticket_vault_subdir.trim().trim_matches('/');
    if sub.is_empty() {
        return Ok(());
    }
    let tickets_dir = vault_root.join(&ns).join(sub);
    fs::create_dir_all(&tickets_dir)?;
    let fname = format!("commit-{}.md", meta.short);
    let path = tickets_dir.join(&fname);
    let marker = format!("<!-- joicy:commit:{} -->", meta.full_hash);
    if path.is_file() {
        let existing = fs::read_to_string(&path)?;
        if existing.contains(&marker) {
            return Ok(());
        }
    }
    let body = format!(
        "---\njoicy: true\njoicy_kind: ticket\njoicy_commit_short: \"{}\"\njoicy_commit_hash: \"{}\"\njoicy_namespace: \"{}\"\n---\n\n\
         # `{}` — {}\n\n\
         - **Date:** {}\n\
         - **Hash:** `{}`\n\
         - Changelog: repository file `CHANGELOG.md` (repo root).\n\
         - Memory / MCP: search label `git/commits/{}`.\n\n\
         {}\n",
        yaml_escape(&meta.short),
        yaml_escape(&meta.full_hash),
        yaml_escape(&ns),
        meta.short,
        meta.subject,
        meta.date_iso,
        meta.full_hash,
        meta.short,
        marker
    );
    fs::write(&path, body)?;
    Ok(())
}
