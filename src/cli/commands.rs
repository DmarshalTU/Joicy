//! CLI command implementations

use crate::config::AppConfig;
use crate::error::{Error, Result};
use crate::memory::CodeContext;
use crate::utils::timestamp;
use crate::workspace::{self, JOICY_CONFIG_LEGACY, JOICY_CONFIG_TOML};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// Initialize a new memory bank in the specified repository path.
pub fn init(path: &str) -> Result<()> {
    let repo_path = PathBuf::from(path);

    if !repo_path.exists() {
        return Err(Error::Config(format!("Path does not exist: {path}")));
    }

    if !repo_path.is_dir() {
        return Err(Error::Config(format!("Path is not a directory: {path}")));
    }

    let joicy_dir = repo_path.join(".joicy");
    fs::create_dir_all(&joicy_dir).map_err(|e| {
        Error::Config(format!("Failed to create .joicy directory: {e}"))
    })?;

    let memory_dir = joicy_dir.join("memory");
    fs::create_dir_all(&memory_dir).map_err(|e| {
        Error::Config(format!("Failed to create memory directory: {e}"))
    })?;

    let config_primary = joicy_dir.join(JOICY_CONFIG_TOML);
    let config_legacy = joicy_dir.join(JOICY_CONFIG_LEGACY);

    if config_primary.is_file() {
        read_and_validate_config(&config_primary)?;
        println!(
            "✓ Using existing configuration from: {}",
            config_primary.display()
        );
    } else if config_legacy.is_file() {
        read_and_validate_config(&config_legacy)?;
        println!(
            "✓ Using existing configuration from: {}",
            config_legacy.display()
        );
        eprintln!(
            "Note: `.joicy/{}` is legacy; rename to `{}` to avoid confusion with other `config.toml` files.",
            JOICY_CONFIG_LEGACY, JOICY_CONFIG_TOML
        );
    } else {
        let mut app_config = AppConfig::default();
        app_config.memory.backend = "sqlite".to_string();
        app_config.memory.path = fs::canonicalize(&memory_dir).unwrap_or(memory_dir.clone());

        let config_str = toml::to_string(&app_config).map_err(|e| {
            Error::Serialization(format!("Failed to serialize default configuration: {e}"))
        })?;
        fs::write(&config_primary, config_str).map_err(|e| {
            Error::Config(format!("Failed to write configuration file: {e}"))
        })?;
        println!("✓ Created new configuration at: {}", config_primary.display());
    }

    append_joicy_gitignore_if_git_repo(&repo_path)?;

    println!(
        "✓ Memory bank initialized successfully at: {}",
        memory_dir.display()
    );
    println!("Next:  joicy hooks install");
    println!("       (optional) export JOICY_VAULT_ROOT=/path/to/one/Obsidian-vault for all repos");
    Ok(())
}

fn read_and_validate_config(path: &Path) -> Result<()> {
    let config_str = fs::read_to_string(path).map_err(|e| {
        Error::Config(format!("Failed to read configuration file: {e}"))
    })?;
    let _: AppConfig = toml::from_str(&config_str).map_err(|e| {
        Error::Config(format!("Invalid configuration file format: {e}"))
    })?;
    Ok(())
}

const GITIGNORE_SENTINEL: &str = "# joicy: local machine memory (added by `joicy init`)";

fn git_work_tree_root(path: &Path) -> Option<PathBuf> {
    let cwd = path.to_str()?;
    let out = std::process::Command::new("git")
        .args(["-C", cwd, "rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(PathBuf::from(t))
    }
}

fn append_joicy_gitignore_if_git_repo(repo_path: &Path) -> Result<()> {
    let Some(root) = git_work_tree_root(repo_path) else {
        return Ok(());
    };
    let gitignore = root.join(".gitignore");
    let block = format!("{GITIGNORE_SENTINEL}\n.joicy/memory/\n");
    if gitignore.is_file() {
        let existing = fs::read_to_string(&gitignore).map_err(|e| {
            Error::Config(format!("Failed to read {}: {e}", gitignore.display()))
        })?;
        if existing.contains(GITIGNORE_SENTINEL) {
            return Ok(());
        }
        let mut out = existing;
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
        out.push_str(&block);
        fs::write(&gitignore, out).map_err(|e| {
            Error::Config(format!("Failed to update {}: {e}", gitignore.display()))
        })?;
    } else {
        fs::write(&gitignore, format!("{block}\n")).map_err(|e| {
            Error::Config(format!("Failed to write {}: {e}", gitignore.display()))
        })?;
    }
    println!("✓ Registered `.joicy/memory/` in {}", gitignore.display());
    Ok(())
}

/// Add text or a file into the local memory bank (FTS index; local POC).
pub fn add(
    text: Option<String>,
    file: Option<PathBuf>,
    label: Option<String>,
    language: String,
) -> Result<()> {
    match (&text, &file) {
        (None, None) => {
            return Err(Error::Config(
                "Provide either --text \"...\" or --file path/to/file".to_string(),
            ));
        }
        (Some(_), Some(_)) => {
            return Err(Error::Config(
                "Use only one of --text or --file, not both".to_string(),
            ));
        }
        _ => {}
    }

    let repo_root = workspace::find_joicy_root()?;
    let cfg = workspace::load_repo_config(&repo_root)?;
    let mut bank = workspace::open_bank(&repo_root, &cfg)?;

    let (content, file_path) = if let Some(t) = text {
        let fp = label.unwrap_or_else(|| "snippet".to_string());
        (t, fp)
    } else {
        let fp = file.expect("checked");
        let content = fs::read_to_string(&fp).map_err(|e| {
            Error::Config(format!("Failed to read file {}: {e}", fp.display()))
        })?;
        let logical = label.unwrap_or_else(|| {
            fp.file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "file".to_string())
        });
        (content, logical)
    };

    let ctx = CodeContext {
        content,
        file_path,
        language,
        metadata: Vec::new(),
        timestamp: timestamp(),
    };
    bank.store(ctx)?;
    println!("✓ Stored 1 entry in local memory bank");
    Ok(())
}

/// Search the local memory bank (FTS5 full-text; not semantic embeddings yet).
pub fn search(query: &str, file: Option<&str>, limit: usize) -> Result<()> {
    let repo_root = workspace::find_joicy_root()?;
    let cfg = workspace::load_repo_config(&repo_root)?;
    let bank = workspace::open_bank(&repo_root, &cfg)?;

    let hits = bank.search_filtered(query, file, limit)?;
    if hits.is_empty() {
        println!("No results.");
        return Ok(());
    }
    for (i, h) in hits.iter().enumerate() {
        let preview: String = h
            .content
            .chars()
            .take(200)
            .collect::<String>()
            .replace('\n', " ");
        println!(
            "---\n{} | {} | {}\n{}\n",
            i + 1,
            h.file_path,
            h.language,
            preview
        );
    }
    Ok(())
}

/// Sync local memory with central (not implemented; reserved for team POC).
pub fn sync(force: bool) -> Result<()> {
    eprintln!(
        "joicy sync: central / team sync is not implemented in the local POC (force={force}).\n\
         Local workflow: `joicy add` / commits + hooks → SQLite + vault + CHANGELOG + tickets; \
         `joicy export`, `joicy changelog show`, MCP `memory_*`."
    );
    Ok(())
}

/// Print local memory bank statistics.
pub fn status() -> Result<()> {
    let repo_root = workspace::find_joicy_root()?;
    let cfg = workspace::load_repo_config(&repo_root)?;
    let bank = workspace::open_bank(&repo_root, &cfg)?;
    let stats = bank.stats()?;
    let mem_dir = workspace::resolve_memory_dir(&repo_root, &cfg);
    println!("Memory bank status:");
    println!("  Backend:     {}", cfg.memory.backend);
    println!("  Data path:   {}", mem_dir.display());
    println!("  Entries:     {}", stats.total_entries);
    println!("  DB size:     {} bytes", stats.storage_size);
    Ok(())
}

/// Remove entries older than `days` (SQLite FTS store only).
pub fn clean(days: u64) -> Result<()> {
    let repo_root = workspace::find_joicy_root()?;
    let cfg = workspace::load_repo_config(&repo_root)?;
    let mut bank = workspace::open_bank(&repo_root, &cfg)?;
    let now = timestamp();
    let cutoff = now.saturating_sub(days.saturating_mul(86_400));
    let removed = bank.purge_before(cutoff)?;
    println!("Removed {removed} entries older than {days} days (cutoff unix {cutoff})");
    Ok(())
}

fn write_vault_notes(
    entries: &[crate::memory::CodeContext],
    target: &Path,
    ns: &str,
) -> Result<usize> {
    fs::create_dir_all(target).map_err(|e| {
        Error::Config(format!("Failed to create vault directory {}: {e}", target.display()))
    })?;

    let mut written = 0usize;
    for ctx in entries.iter() {
        let slug = crate::vault_markdown::slugify_label(&ctx.file_path);
        let mut hasher = DefaultHasher::new();
        ctx.content.hash(&mut hasher);
        ctx.file_path.hash(&mut hasher);
        ctx.timestamp.hash(&mut hasher);
        let h = hasher.finish();
        let fname = format!("{}_{}_{:x}.md", ctx.timestamp, slug, h);
        let path = target.join(fname);

        let mut front = String::from("---\n");
        front.push_str("joicy: true\n");
        front.push_str(&crate::vault_markdown::yaml_string_field(
            "joicy_label",
            &ctx.file_path,
        ));
        front.push_str(&crate::vault_markdown::yaml_string_field(
            "joicy_language",
            &ctx.language,
        ));
        front.push_str(&crate::vault_markdown::yaml_string_field(
            "joicy_namespace",
            ns,
        ));
        front.push_str(&format!("joicy_timestamp: {}\n", ctx.timestamp));
        front.push_str("---\n\n");
        let body = crate::vault_markdown::format_note_body(&ctx.language, &ctx.content);
        let doc = format!("{front}{body}");
        fs::write(&path, doc).map_err(|e| {
            Error::Config(format!("Failed to write {}: {e}", path.display()))
        })?;
        written += 1;
    }
    Ok(written)
}

fn vault_export_with_cfg(
    repo_root: &Path,
    cfg: &AppConfig,
    dir: Option<&Path>,
    namespace: Option<&str>,
    limit: usize,
) -> Result<()> {
    let bank = workspace::open_bank(repo_root, cfg)?;
    let entries = bank.dump_entries(limit)?;
    let (vault_root, ns) = workspace::resolve_vault_paths(repo_root, cfg, dir, namespace);
    let target = vault_root.join(&ns);
    let written = write_vault_notes(&entries, &target, &ns)?;
    println!(
        "✓ Wrote {written} markdown note(s) under {}",
        target.display()
    );
    println!("  Open this folder (or its parent vault root) in Obsidian as a vault.");
    Ok(())
}

/// Export memory entries as Markdown files for Obsidian (and similar).
///
/// Vault root resolution: CLI `--dir` → `[vault] export_root` in config → `JOICY_VAULT_ROOT` → `.joicy/vault`.
/// Namespace: `--namespace` → `[vault] namespace` → repository folder name.
pub fn vault_export(dir: Option<&Path>, namespace: Option<&str>, limit: usize) -> Result<()> {
    let repo_root = workspace::find_joicy_root()?;
    let cfg = workspace::load_repo_config(&repo_root)?;
    vault_export_with_cfg(&repo_root, &cfg, dir, namespace, limit)
}

/// Capture `HEAD` into the memory bank and run the **four-way** local trigger (post-commit hook).
///
/// 1. SQLite + FTS · 2. Vault markdown export · 3. `CHANGELOG.md` line · 4. ticket stub in vault.
#[cfg(feature = "git")]
pub fn automation_on_commit() -> Result<()> {
    let repo_root = workspace::find_joicy_root()?;
    let cfg = workspace::load_repo_config(&repo_root)?;
    let (ctx, meta) = crate::git::capture_head_for_pipeline(&repo_root)?;
    let mut bank = workspace::open_bank(&repo_root, &cfg)?;
    if ctx.file_path.starts_with("git/commits/") {
        bank.delete_by_file_path(&ctx.file_path)?;
    }
    bank.store(ctx)?;
    println!("✓ Joicy [1/4] captured HEAD → SQLite FTS");
    if cfg.vault.auto_export {
        vault_export_with_cfg(&repo_root, &cfg, None, None, 10_000)?;
        println!("✓ Joicy [2/4] refreshed Obsidian vault export");
    } else {
        println!("○ Joicy [2/4] vault export skipped (vault.auto_export = false)");
    }
    if cfg.automation.changelog_on_commit {
        crate::automation::append_changelog_entry(&repo_root, &cfg, &meta)?;
        println!(
            "✓ Joicy [3/4] changelog → {}",
            crate::automation::changelog_file_path(&repo_root, &cfg).display()
        );
    } else {
        println!("○ Joicy [3/4] changelog skipped (automation.changelog_on_commit = false)");
    }
    if cfg.automation.ticket_note_on_commit && cfg.vault.auto_export {
        crate::automation::write_ticket_stub(&repo_root, &cfg, &meta)?;
        let (vr, ns) = workspace::resolve_vault_paths(&repo_root, &cfg, None, None);
        let td = vr
            .join(&ns)
            .join(cfg.automation.ticket_vault_subdir.trim().trim_matches('/'));
        println!(
            "✓ Joicy [4/4] ticket stub → {}",
            td.join(format!("commit-{}.md", meta.short)).display()
        );
    } else {
        println!(
            "○ Joicy [4/4] ticket stub skipped (needs automation.ticket_note_on_commit + vault.auto_export)"
        );
    }
    Ok(())
}

/// Print the tail of the auto-maintained changelog.
pub fn changelog_show(lines: usize) -> Result<()> {
    let repo_root = workspace::find_joicy_root()?;
    let cfg = workspace::load_repo_config(&repo_root)?;
    let tail = crate::automation::read_changelog_tail(&repo_root, &cfg, lines)?;
    println!("{}", tail.trim_end());
    Ok(())
}

/// Stub when built without `git`.
#[cfg(not(feature = "git"))]
pub fn automation_on_commit() -> Result<()> {
    Err(Error::Config(
        "automation on-commit requires the `git` feature (rebuild with default features)."
            .to_string(),
    ))
}

/// Install `post-commit` hook (`joicy automation on-commit`).
#[cfg(feature = "git")]
pub fn hooks_install() -> Result<()> {
    let repo_root = workspace::find_joicy_root()?;
    crate::git::install_post_commit_hook(&repo_root)?;
    println!("Each commit runs: joicy automation on-commit");
    println!("  (1) SQLite FTS  (2) vault export  (3) CHANGELOG  (4) ticket stub under vault/tickets/");
    println!("Set one shared Obsidian vault for every repo, e.g.:");
    println!("  export JOICY_VAULT_ROOT=\"$HOME/Obsidian/JoicyMemory\"");
    println!("Or add to .joicy/joicy.toml:  [vault]  export_root = \"/path/to/vault\"");
    Ok(())
}

#[cfg(not(feature = "git"))]
pub fn hooks_install() -> Result<()> {
    Err(Error::Config(
        "hooks install requires the `git` feature (rebuild with default features).".into(),
    ))
}

/// Export entries as JSON (newest first; SQLite only).
pub fn export(output: Option<&str>) -> Result<()> {
    let repo_root = workspace::find_joicy_root()?;
    let cfg = workspace::load_repo_config(&repo_root)?;
    let bank = workspace::open_bank(&repo_root, &cfg)?;
    let entries = bank.dump_entries(1_000_000)?;
    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| Error::Serialization(format!("export json: {e}")))?;
    let output_path = output.unwrap_or("joicy-export.json");
    fs::write(output_path, json).map_err(Error::Io)?;
    println!("✓ Exported {} entries to {output_path}", entries.len());
    Ok(())
}
