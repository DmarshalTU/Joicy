//! Capture the current `HEAD` commit into a [`CodeContext`](crate::memory::CodeContext) using the `git` CLI.

use crate::error::{Error, Result};
use crate::memory::CodeContext;
use crate::utils::timestamp;
use std::path::Path;
use std::process::Command;

const MAX_PATCH_BYTES: usize = 120_000;

/// Resolve the git work tree root for a directory inside the repo.
pub fn git_workdir(from: &Path) -> Result<std::path::PathBuf> {
    let out = Command::new("git")
        .arg("-C")
        .arg(from)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| Error::Git(format!("failed to run git: {e}")))?;
    if !out.status.success() {
        return Err(Error::Git(
            "not a git repository (run `git init` or use a repo checkout)".into(),
        ));
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(std::path::PathBuf::from(s))
}

/// Build a memory entry for `HEAD` (message + patch, truncated).
pub fn capture_head_context(from: &Path) -> Result<CodeContext> {
    let top = git_workdir(from)?;

    let full_hash = {
        let o = Command::new("git")
            .current_dir(&top)
            .args(["rev-parse", "HEAD"])
            .output()
            .map_err(|e| Error::Git(format!("git rev-parse failed: {e}")))?;
        if !o.status.success() {
            return Err(Error::Git("git rev-parse HEAD failed".into()));
        }
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    };

    let short = {
        let o = Command::new("git")
            .current_dir(&top)
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .map_err(|e| Error::Git(format!("git rev-parse --short failed: {e}")))?;
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    };

    let message = {
        let o = Command::new("git")
            .current_dir(&top)
            .args(["log", "-1", "--pretty=format:%s%n%n%b"])
            .output()
            .map_err(|e| Error::Git(format!("git log failed: {e}")))?;
        String::from_utf8_lossy(&o.stdout).to_string()
    };

    let mut patch = {
        let o = Command::new("git")
            .current_dir(&top)
            .args(["show", "-1", "--no-color", "--patch", "HEAD"])
            .output()
            .map_err(|e| Error::Git(format!("git show failed: {e}")))?;
        String::from_utf8_lossy(&o.stdout).to_string()
    };

    if patch.len() > MAX_PATCH_BYTES {
        patch.truncate(MAX_PATCH_BYTES);
        patch.push_str("\n\n… (truncated for Joicy memory bank)\n");
    }

    let content = format!(
        "# Commit {short}\n\nFull hash: `{full_hash}`\n\n## Message\n\n{message}\n\n## Patch (git show)\n\n{patch}\n"
    );

    Ok(CodeContext {
        content,
        file_path: format!("git/commits/{short}"),
        language: "git".to_string(),
        metadata: vec![
            ("hash".to_string(), full_hash),
            ("short".to_string(), short.clone()),
        ],
        timestamp: timestamp(),
    })
}

/// Metadata for changelog lines and ticket stubs (post-commit pipeline).
#[derive(Debug, Clone)]
pub struct CommitMeta {
    /// First line of commit message (`%s`).
    pub subject: String,
    /// Author date, ISO-8601 (`%cI`).
    pub date_iso: String,
    /// Abbreviated object name (`git rev-parse --short HEAD`).
    pub short: String,
    /// Full commit hash.
    pub full_hash: String,
}

fn git_log_format(top: &Path, pretty: &str) -> Result<String> {
    let arg = format!("--pretty=format:{pretty}");
    let o = Command::new("git")
        .current_dir(top)
        .args(["log", "-1", arg.as_str()])
        .output()
        .map_err(|e| Error::Git(format!("git log failed: {e}")))?;
    if !o.status.success() {
        return Err(Error::Git("git log -1 failed".into()));
    }
    Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// Full capture for the post-commit pipeline: memory row + changelog / ticket fields.
pub fn capture_head_for_pipeline(from: &Path) -> Result<(CodeContext, CommitMeta)> {
    let ctx = capture_head_context(from)?;
    let top = git_workdir(from)?;
    let subject = git_log_format(&top, "%s")?;
    let date_iso = git_log_format(&top, "%cI")?;
    let short = ctx
        .metadata
        .iter()
        .find(|(k, _)| k == "short")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| "?".into());
    let full_hash = ctx
        .metadata
        .iter()
        .find(|(k, _)| k == "hash")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| "?".into());
    Ok((
        ctx,
        CommitMeta {
            subject,
            date_iso,
            short,
            full_hash,
        },
    ))
}
