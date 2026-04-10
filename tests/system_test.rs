//! Subprocess tests against the real `joicy` binary (`CARGO_BIN_EXE_joicy`).

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn joicy_exe() -> &'static str {
    env!("CARGO_BIN_EXE_joicy")
}

fn run(args: &[&str], cwd: &Path) -> std::process::Output {
    Command::new(joicy_exe())
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn {}: {e}", joicy_exe()))
}

#[test]
fn cli_version_reports_features() {
    let tmp = TempDir::new().expect("tempdir");
    let o = run(&["--version"], tmp.path());
    assert!(o.status.success(), "stderr={}", String::from_utf8_lossy(&o.stderr));
    let s = String::from_utf8_lossy(&o.stdout);
    assert!(
        s.contains("features:"),
        "expected feature stamp in version, got {s:?}"
    );
}

#[test]
fn cli_init_add_search_status_export_vault_changelog_sync() {
    let tmp = TempDir::new().expect("tempdir");
    let p = tmp.path();

    assert!(run(&["init", "."], p).status.success());
    assert!(run(&["add", "--text", "system test phrase", "--label", "cli_sys"], p).status.success());

    let search = run(&["search", "phrase", "--limit", "5"], p);
    assert!(search.status.success());
    let search_out = String::from_utf8_lossy(&search.stdout);
    assert!(
        search_out.contains("cli_sys") || search_out.contains("phrase"),
        "search output: {search_out}"
    );

    assert!(run(&["status"], p).status.success());

    assert!(run(&["export", "-o", "dump.json"], p).status.success());
    assert!(p.join("dump.json").is_file());

    assert!(run(&["vault", "export", "-l", "50"], p).status.success());

    let cl = run(&["changelog", "show", "-n", "10"], p);
    assert!(cl.status.success());
    let cl_out = String::from_utf8_lossy(&cl.stdout);
    assert!(
        cl_out.contains("no changelog file") || cl_out.contains("Changelog"),
        "changelog show: {cl_out}"
    );

    let sy = run(&["sync"], p);
    assert!(sy.status.success());
    let sy_err = String::from_utf8_lossy(&sy.stderr);
    assert!(
        sy_err.contains("central") || sy_err.contains("POC"),
        "sync stderr: {sy_err}"
    );
}

#[test]
fn cli_git_post_commit_four_way_smoke() {
    if Command::new("git")
        .arg("--version")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
    {
        return;
    }

    let tmp = TempDir::new().expect("tempdir");
    let p = tmp.path();

    assert!(
        Command::new("git")
            .arg("init")
            .current_dir(p)
            .status()
            .expect("git")
            .success()
    );
    assert!(
        Command::new("git")
            .args(["config", "user.email", "sys@test.local"])
            .current_dir(p)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args(["config", "user.name", "System Test"])
            .current_dir(p)
            .status()
            .unwrap()
            .success()
    );

    assert!(run(&["init", "."], p).status.success());
    assert!(run(&["hooks", "install"], p).status.success());

    std::fs::write(p.join("f.txt"), "x").expect("write");
    assert!(
        Command::new("git")
            .args(["add", "f.txt"])
            .current_dir(p)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args(["commit", "-m", "system test commit"])
            .current_dir(p)
            .status()
            .unwrap()
            .success()
    );

    let log = fs::read_to_string(p.join("CHANGELOG.md")).expect("CHANGELOG");
    assert!(
        log.contains("joicy:commit:"),
        "CHANGELOG should record commit marker"
    );
    assert!(
        log.contains("system test commit"),
        "CHANGELOG should list subject"
    );

    let ns = p
        .file_name()
        .and_then(|n| n.to_str())
        .expect("tmp name");
    let ticket_dir = p.join(".joicy").join("vault").join(ns).join("tickets");
    let n_tickets = fs::read_dir(&ticket_dir)
        .map(|d| d.filter_map(|e| e.ok()).count())
        .unwrap_or(0);
    assert!(
        n_tickets > 0,
        "expected ticket stub under {}",
        ticket_dir.display()
    );
}
