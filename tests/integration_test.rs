//! Integration tests for Joicy

use joicy::*;
use joicy::workspace;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use tempfile::TempDir;

/// `find_joicy_root` uses the process cwd; integration tests must not change cwd in parallel.
static CWD_LOCK: Mutex<()> = Mutex::new(());

fn with_cwd<T>(path: &Path, f: impl FnOnce() -> T) -> T {
    let _guard = CWD_LOCK.lock().expect("cwd lock");
    let prev = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(path).expect("chdir");
    let out = f();
    std::env::set_current_dir(prev).expect("restore cwd");
    out
}

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
    assert!(config.vault.auto_export);
    assert!(config.automation.changelog_on_commit);
    assert!(config.automation.ticket_note_on_commit);
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
    
    let config_file = joicy_dir.join(workspace::JOICY_CONFIG_TOML);
    assert!(config_file.exists(), "joicy.toml should exist");
    assert!(config_file.is_file(), "joicy.toml should be a file");
    
    // Verify config file can be parsed
    let config_str = fs::read_to_string(&config_file).expect("Should read config file");
    let config: config::AppConfig = toml::from_str(&config_str)
        .expect("Config file should be valid TOML");
    assert_eq!(config.memory.backend, "sqlite");
}

#[test]
fn test_vault_markdown_write_agent_note() {
    let temp_dir = TempDir::new().expect("temp dir");
    let test_path = temp_dir.path();
    joicy::cli::init(test_path.to_str().unwrap()).expect("init");
    let cfg = workspace::load_repo_config(test_path).expect("cfg");
    let path = joicy::vault_markdown::write_vault_markdown_file(
        test_path,
        &cfg,
        joicy::vault_markdown::VaultMarkdownWrite {
            logical_label: "notes/mcp-style-note",
            body: "Body line for vault test.",
            language: "md",
            subfolder: Some("notes"),
            source: "integration-test",
            file_stem: None,
        },
    )
    .expect("write vault file");
    assert!(path.is_file(), "expected {}", path.display());
    let text = fs::read_to_string(&path).expect("read note");
    assert!(text.contains("joicy: true"));
    assert!(text.contains("joicy_source"));
    assert!(text.contains("Body line for vault test."));
    assert!(
        path.to_string_lossy().contains("notes"),
        "path should include notes subfolder: {}",
        path.display()
    );
}

#[test]
fn test_vault_markdown_stable_file_stem() {
    let temp_dir = TempDir::new().expect("temp dir");
    let test_path = temp_dir.path();
    joicy::cli::init(test_path.to_str().unwrap()).expect("init");
    let cfg = workspace::load_repo_config(test_path).expect("cfg");
    let path = joicy::vault_markdown::write_vault_markdown_file(
        test_path,
        &cfg,
        joicy::vault_markdown::VaultMarkdownWrite {
            logical_label: "notes/stable",
            body: "linked body",
            language: "md",
            subfolder: Some("notes"),
            source: "integration-test",
            file_stem: Some("Stable-Stem-Note"),
        },
    )
    .expect("write");
    assert!(
        path.ends_with("Stable-Stem-Note.md"),
        "expected stable name, got {}",
        path.display()
    );
}

#[test]
fn test_local_poc_add_search_status() {
    let temp_dir = TempDir::new().expect("temp dir");
    let test_path = temp_dir.path();

    joicy::cli::init(test_path.to_str().unwrap()).expect("init");

    with_cwd(test_path, || {
        joicy::cli::add(
            Some("use anyhow::Result for CLI binaries".to_string()),
            None,
            Some("src/main.rs".to_string()),
            "rust".to_string(),
        )
        .expect("add");
    });

    let mem_dir = test_path.join(".joicy").join("memory");
    let bank = joicy::memory::open_local_memory_bank(mem_dir.as_path()).expect("open bank");
    let hits = bank
        .search_filtered("anyhow", None, 10)
        .expect("search");
    assert_eq!(hits.len(), 1);
    assert!(hits[0].content.contains("anyhow"));
}

#[test]
fn test_vault_export_writes_markdown() {
    let temp_dir = TempDir::new().expect("temp dir");
    let test_path = temp_dir.path();

    joicy::cli::init(test_path.to_str().unwrap()).expect("init");

    with_cwd(test_path, || {
        joicy::cli::add(
            Some("vault export body line".to_string()),
            None,
            Some("notes/idea.md".to_string()),
            "md".to_string(),
        )
        .expect("add");
        joicy::cli::vault_export(None, None, 100).expect("vault export");
    });

    let vault_ns = test_path
        .file_name()
        .and_then(|n| n.to_str())
        .expect("temp name");
    let vault_dir = test_path
        .join(".joicy")
        .join("vault")
        .join(vault_ns);
    let mut found = false;
    for e in fs::read_dir(&vault_dir).expect("read vault dir") {
        let e = e.expect("entry");
        let p = e.path();
        if p.extension().map(|x| x == "md").unwrap_or(false) {
            let s = fs::read_to_string(&p).expect("read md");
            assert!(s.contains("joicy: true"));
            assert!(s.contains("vault export body line"));
            found = true;
            break;
        }
    }
    assert!(found, "expected at least one .md under {}", vault_dir.display());
}

#[test]
fn test_automation_on_commit_after_git_commit() {
    if Command::new("git")
        .arg("--version")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
    {
        return;
    }

    let temp = TempDir::new().expect("temp");
    let p = temp.path();
    assert!(
        Command::new("git")
            .arg("init")
            .current_dir(p)
            .status()
            .expect("git init")
            .success()
    );
    assert!(
        Command::new("git")
            .args(["config", "user.email", "joicy@test.local"])
            .current_dir(p)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args(["config", "user.name", "Joicy Test"])
            .current_dir(p)
            .status()
            .unwrap()
            .success()
    );

    joicy::cli::init(p.to_str().unwrap()).expect("joicy init");
    fs::write(p.join("tracked.txt"), "hello automation").expect("write");

    with_cwd(p, || {
        assert!(
            Command::new("git")
                .args(["add", "."])
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .args(["commit", "-m", "first commit for joicy"])
                .status()
                .unwrap()
                .success()
        );
        joicy::cli::automation_on_commit().expect("automation");
    });

    let mem = p.join(".joicy").join("memory");
    let bank = joicy::memory::open_local_memory_bank(mem.as_path()).expect("bank");
    let hits = bank
        .search_filtered("joicy", None, 20)
        .expect("search");
    assert!(
        !hits.is_empty(),
        "expected commit capture to be searchable"
    );

    let ns = p
        .file_name()
        .and_then(|n| n.to_str())
        .expect("temp name");
    let vault_dir = p.join(".joicy").join("vault").join(ns);
    let md_count = fs::read_dir(&vault_dir)
        .expect("vault dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
        .count();
    assert!(md_count >= 1, "vault export should write at least one .md");

    let changelog = fs::read_to_string(p.join("CHANGELOG.md")).expect("CHANGELOG.md");
    assert!(
        changelog.contains("joicy:commit:"),
        "changelog should contain joicy commit marker"
    );
    assert!(
        changelog.contains("first commit for joicy"),
        "changelog should list commit subject"
    );

    let tickets_dir = vault_dir.join("tickets");
    let ticket_count = fs::read_dir(&tickets_dir)
        .expect("tickets dir")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("commit-") && n.ends_with(".md"))
                .unwrap_or(false)
        })
        .count();
    assert!(ticket_count >= 1, "expected at least one ticket stub under vault/tickets/");
}

#[test]
fn test_automation_on_commit_twice_same_head_no_extra_rows() {
    if Command::new("git")
        .arg("--version")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
    {
        return;
    }

    let temp = TempDir::new().expect("temp");
    let p = temp.path();
    assert!(
        Command::new("git")
            .arg("init")
            .current_dir(p)
            .status()
            .expect("git init")
            .success()
    );
    assert!(
        Command::new("git")
            .args(["config", "user.email", "idempotent@local"])
            .current_dir(p)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(p)
            .status()
            .unwrap()
            .success()
    );

    joicy::cli::init(p.to_str().unwrap()).expect("joicy init");
    fs::write(p.join("t.txt"), "idempotent").expect("write");

    fn entry_count(repo: &std::path::Path) -> usize {
        let cfg = workspace::load_repo_config(repo).expect("cfg");
        let bank = workspace::open_bank(repo, &cfg).expect("bank");
        bank.stats().expect("stats").total_entries
    }

    with_cwd(p, || {
        assert!(Command::new("git").args(["add", "."]).status().unwrap().success());
        assert!(
            Command::new("git")
                .args(["commit", "-m", "only commit"])
                .status()
                .unwrap()
                .success()
        );
        joicy::cli::automation_on_commit().expect("auto1");
    });

    let n1 = entry_count(p);
    assert!(n1 >= 1);

    with_cwd(p, || {
        joicy::cli::automation_on_commit().expect("auto2");
    });

    let n2 = entry_count(p);
    assert_eq!(
        n1, n2,
        "re-running automation for same HEAD must not duplicate FTS rows"
    );
}

#[test]
fn test_find_joicy_root_legacy_config_only() {
    let temp_dir = TempDir::new().expect("temp");
    let p = temp_dir.path();
    let jd = p.join(".joicy");
    fs::create_dir_all(jd.join("memory")).expect("dirs");
    let cfg = config::AppConfig::default();
    let s = toml::to_string(&cfg).expect("toml");
    fs::write(jd.join(workspace::JOICY_CONFIG_LEGACY), s).expect("write legacy config");

    let root = workspace::find_joicy_root_from(p).expect("find root");
    assert_eq!(root, p);
    let loaded = workspace::load_repo_config(p).expect("load");
    assert_eq!(loaded.memory.backend, "sqlite");
}

#[test]
fn test_init_appends_gitignore_when_inside_git_repo() {
    if Command::new("git")
        .arg("--version")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
    {
        return;
    }

    let temp = TempDir::new().expect("temp");
    let p = temp.path();
    assert!(
        Command::new("git")
            .arg("init")
            .current_dir(p)
            .status()
            .expect("git init")
            .success()
    );

    joicy::cli::init(p.to_str().unwrap()).expect("joicy init");

    let gi = fs::read_to_string(p.join(".gitignore")).expect("gitignore");
    assert!(
        gi.contains(".joicy/memory/"),
        "expected .joicy/memory/ in gitignore: {gi:?}"
    );
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

