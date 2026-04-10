//! YAML front matter and markdown bodies for Obsidian vault files (export + MCP agent notes).

use crate::config::AppConfig;
use crate::error::{Error, Result};
use crate::utils::timestamp;
use crate::workspace;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// YAML string field line for front matter (`key: "escaped"`).
pub fn yaml_string_field(key: &str, value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\n', '\r'], " ");
    format!("{key}: \"{escaped}\"\n")
}

/// File-name slug from a label (alphanumeric + `_` `-`).
pub fn slugify_label(s: &str) -> String {
    let t: String = s
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(72)
        .collect();
    let t = t.trim_matches('_').to_string();
    if t.is_empty() {
        "note".to_string()
    } else {
        t
    }
}

/// Wrap content in a fenced block when language is a code type.
pub fn format_note_body(language: &str, content: &str) -> String {
    let lang = language.trim();
    if lang.is_empty()
        || lang == "text"
        || lang == "md"
        || lang == "markdown"
        || lang == "git"
    {
        content.to_string()
    } else {
        format!("```{lang}\n{content}\n```\n")
    }
}

/// Sanitize a user-provided file stem for `Note.md` (no path components, safe characters only).
pub fn sanitize_vault_file_stem(raw: &str) -> Option<String> {
    let s = raw.trim();
    if s.is_empty() || s == "." || s == ".." {
        return None;
    }
    let base = Path::new(s)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(s)
        .trim();
    let base = base
        .strip_suffix(".md")
        .or_else(|| base.strip_suffix(".MD"))
        .unwrap_or(base)
        .trim();
    if base.is_empty() {
        return None;
    }
    let safe: String = base
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .take(120)
        .collect();
    let safe = safe.trim_matches('_').trim().to_string();
    if safe.is_empty() {
        None
    } else {
        Some(safe)
    }
}

/// Parameters for [`write_vault_markdown_file`].
pub struct VaultMarkdownWrite<'a> {
    /// Front matter `joicy_label`, e.g. `notes/standup-2026-04-10`.
    pub logical_label: &'a str,
    /// Note body (markdown unless `language` is a code fence type).
    pub body: &'a str,
    /// `md`, `text`, or a language tag for fenced blocks.
    pub language: &'a str,
    /// Directory under the vault namespace; default applied when `None` or empty inside writer.
    pub subfolder: Option<&'a str>,
    /// How the file was created (`mcp`, `export`, …).
    pub source: &'a str,
    /// If set (e.g. `Graph-Hub`), file is `Graph-Hub.md` for Obsidian `[[Graph-Hub]]` / graph.
    pub file_stem: Option<&'a str>,
}

/// Write one markdown note under `{vault_root}/{namespace}/{subfolder}/` (default `notes/`).
pub fn write_vault_markdown_file(
    repo_root: &Path,
    cfg: &AppConfig,
    w: VaultMarkdownWrite<'_>,
) -> Result<PathBuf> {
    let (vault_root, ns) = workspace::resolve_vault_paths(repo_root, cfg, None, None);
    let sub = w
        .subfolder
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("notes");
    let sub = sub.trim().trim_matches('/');
    let target = if sub.is_empty() {
        vault_root.join(&ns)
    } else {
        vault_root.join(&ns).join(sub)
    };
    fs::create_dir_all(&target).map_err(|e| {
        Error::Config(format!("Failed to create vault directory {}: {e}", target.display()))
    })?;

    let ts = timestamp();
    let fname = if let Some(stem) = w.file_stem.and_then(sanitize_vault_file_stem) {
        format!("{stem}.md")
    } else {
        let slug = slugify_label(w.logical_label);
        let mut hasher = DefaultHasher::new();
        w.body.hash(&mut hasher);
        w.logical_label.hash(&mut hasher);
        ts.hash(&mut hasher);
        let h = hasher.finish();
        format!("{ts}_{slug}_{h:x}.md")
    };
    let path = target.join(&fname);

    let mut front = String::from("---\n");
    front.push_str("joicy: true\n");
    front.push_str(&yaml_string_field("joicy_label", w.logical_label));
    front.push_str(&yaml_string_field("joicy_language", w.language));
    front.push_str(&yaml_string_field("joicy_namespace", &ns));
    front.push_str(&format!("joicy_timestamp: {ts}\n"));
    front.push_str("joicy_kind: note\n");
    front.push_str(&yaml_string_field("joicy_source", w.source));
    front.push_str("---\n\n");
    let formatted = format_note_body(w.language, w.body);
    let doc = format!("{front}{formatted}");
    fs::write(&path, doc).map_err(|e| {
        Error::Config(format!("Failed to write {}: {e}", path.display()))
    })?;
    Ok(path)
}
