//! Install git hooks that run Joicy automation.

use crate::error::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const HOOK_MARKER: &str = "joicy automation on-commit";

fn hooks_dir(repo_root: &Path) -> Result<PathBuf> {
    let wt = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map_err(|e| Error::Git(format!("could not run `git`: {e}")))?;
    if !wt.status.success() {
        return Err(Error::Git(
            "`joicy hooks install` must be run inside a git repository.\n\
             This folder has Joicy (`.joicy/`) but no `.git/` yet — run:\n\
               git init\n\
             then commit at least once if you like, then:\n\
               joicy hooks install"
                .into(),
        ));
    }
    let inside = String::from_utf8_lossy(&wt.stdout).trim().to_string();
    if inside != "true" {
        return Err(Error::Git(
            "`joicy hooks install` must be run inside a git work tree (not a bare repo).".into(),
        ));
    }

    let gd_out = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["rev-parse", "--git-dir"])
        .output()
        .map_err(|e| Error::Git(format!("git rev-parse --git-dir failed: {e}")))?;
    if !gd_out.status.success() {
        return Err(Error::Git(
            "git could not resolve --git-dir (is `git` installed and this a valid repo?)".into(),
        ));
    }
    let mut git_dir = PathBuf::from(String::from_utf8_lossy(&gd_out.stdout).trim());
    if !git_dir.is_absolute() {
        git_dir = repo_root.join(git_dir);
    }
    Ok(git_dir.join("hooks"))
}

/// Install `post-commit` to capture the commit into Joicy and refresh the vault (if enabled).
pub fn install_post_commit_hook(repo_root: &Path) -> Result<()> {
    let hooks = hooks_dir(repo_root)?;
    fs::create_dir_all(&hooks).map_err(|e| {
        Error::Git(format!("failed to create hooks directory {}: {e}", hooks.display()))
    })?;

    let hook_path = hooks.join("post-commit");
    if hook_path.exists() {
        let existing = fs::read_to_string(&hook_path).unwrap_or_default();
        if !existing.contains(HOOK_MARKER) && !existing.trim().is_empty() {
            return Err(Error::Git(format!(
                "{} already exists and is not managed by Joicy; merge manually or remove it first",
                hook_path.display()
            )));
        }
        // If already Joicy-managed, rewrite so `JOICY_EXE` matches this `joicy` binary.
    }

    let joicy_exe = std::env::current_exe().map_err(|e| {
        Error::Git(format!("could not resolve path to `joicy` binary: {e}"))
    })?;
    // Safe for sh single-quoted string: close quote, escape ', reopen
    let q = joicy_exe.display().to_string().replace('\'', "'\"'\"'");
    let script = format!(
        r#"#!/bin/sh
# {}
ROOT=$(git rev-parse --show-toplevel) || exit 0
cd "$ROOT" || exit 0
JOICY_EXE='{q}'
test -x "$JOICY_EXE" || exit 0
"$JOICY_EXE" automation on-commit || true
"#,
        HOOK_MARKER,
        q = q
    );

    fs::write(&hook_path, script).map_err(|e| {
        Error::Git(format!("failed to write {}: {e}", hook_path.display()))
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)
            .map_err(|e| Error::Git(format!("stat hook: {e}")))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms).map_err(|e| Error::Git(format!("chmod hook: {e}")))?;
    }

    println!("✓ Installed {}", hook_path.display());
    Ok(())
}
