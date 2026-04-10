//! MCP server over stdio (`joicy mcp serve`) using `rmcp`.

use crate::memory::CodeContext;
use crate::utils::timestamp;
use crate::workspace;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    serve_server,
    tool, tool_router,
    transport,
};

/// MCP service: tools talk to the same SQLite memory bank as the CLI.
#[derive(Debug, Clone)]
pub struct JoicyMcpServer {
    /// Held for `#[tool_router]`; dispatch uses generated code, not direct reads.
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

#[derive(serde::Deserialize, rmcp::schemars::JsonSchema)]
pub struct MemorySearchParams {
    /// Full-text query (FTS), same as `joicy search QUERY`
    query: String,
    #[serde(default = "default_limit")]
    limit: usize,
    /// Optional substring filter on stored file_path / label
    file_substr: Option<String>,
}

fn default_limit() -> usize {
    10
}

#[derive(serde::Deserialize, rmcp::schemars::JsonSchema)]
pub struct MemoryStoreParams {
    /// Body text to store
    content: String,
    /// Logical path label (default: snippet)
    #[serde(default)]
    file_path: Option<String>,
    #[serde(default = "default_lang")]
    language: String,
}

fn default_lang() -> String {
    "text".to_string()
}

#[derive(serde::Deserialize, rmcp::schemars::JsonSchema)]
pub struct MemoryChangelogParams {
    /// Max lines to return from the end of CHANGELOG.md (or `automation.changelog_path`)
    #[serde(default = "default_changelog_lines")]
    lines: usize,
}

fn default_changelog_lines() -> usize {
    80
}

#[derive(serde::Deserialize, rmcp::schemars::JsonSchema)]
pub struct MemoryVaultNoteParams {
    /// Markdown body. If `title` is set and this does not start with `#`, a `# Title` line is prepended.
    content: String,
    /// Title for the note (used in `joicy_label` and optional heading).
    #[serde(default)]
    title: Option<String>,
    /// Subfolder under the repo namespace in the vault (default `notes`). Use `ideas`, `meetings`, etc.
    #[serde(default)]
    subfolder: Option<String>,
    #[serde(default = "default_vault_note_language")]
    language: String,
    /// Also insert into SQLite FTS so `memory_search` finds this note.
    #[serde(default = "default_index_memory")]
    index_memory: bool,
    /// If set, file is `{file_stem}.md` (sanitized) so `[[file stem]]` works in Obsidian graph. Omit for auto hash filename.
    #[serde(default)]
    file_stem: Option<String>,
}

fn default_vault_note_language() -> String {
    "md".to_string()
}

fn default_index_memory() -> bool {
    true
}

#[tool_router(server_handler)]
impl JoicyMcpServer {
    /// New server with tool router wired for memory, changelog, and vault notes.
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// Search the Joicy memory bank (FTS). Uses `JOICY_REPO_ROOT` when set, else discovers `.joicy` from cwd.
    #[tool(
        name = "memory_search",
        description = "Full-text search the Joicy memory bank for this repository. Optional env: JOICY_REPO_ROOT=/path/to/repo if the process cwd is not the repo root."
    )]
    pub async fn memory_search(
        &self,
        params: Parameters<MemorySearchParams>,
    ) -> Result<String, String> {
        let p = params.0;
        tokio::task::spawn_blocking(move || {
            let root = workspace::resolve_repo_root().map_err(|e| e.to_string())?;
            let cfg = workspace::load_repo_config(&root).map_err(|e| e.to_string())?;
            let bank = workspace::open_bank(&root, &cfg).map_err(|e| e.to_string())?;
            let hits = bank
                .search_filtered(&p.query, p.file_substr.as_deref(), p.limit)
                .map_err(|e| e.to_string())?;
            Ok(format_results(&hits))
        })
        .await
        .map_err(|e| format!("join error: {e}"))?
    }

    /// Append an entry to the memory bank.
    #[tool(
        name = "memory_store",
        description = "Store text in the Joicy SQLite memory bank only. For Obsidian files use `memory_vault_note` (writes markdown under the vault + optional FTS index)."
    )]
    pub async fn memory_store(
        &self,
        params: Parameters<MemoryStoreParams>,
    ) -> Result<String, String> {
        let p = params.0;
        let content = p.content;
        let file_path = p.file_path.unwrap_or_else(|| "snippet".to_string());
        let language = p.language;
        tokio::task::spawn_blocking(move || {
            let root = workspace::resolve_repo_root().map_err(|e| e.to_string())?;
            let cfg = workspace::load_repo_config(&root).map_err(|e| e.to_string())?;
            let mut bank = workspace::open_bank(&root, &cfg).map_err(|e| e.to_string())?;
            let ctx = CodeContext {
                content,
                file_path,
                language,
                metadata: Vec::new(),
                timestamp: timestamp(),
            };
            bank.store(ctx).map_err(|e| e.to_string())?;
            Ok("Stored 1 entry in Joicy memory bank.".to_string())
        })
        .await
        .map_err(|e| format!("join error: {e}"))?
    }

    /// Last lines of the Joicy-managed changelog (same file as post-commit step 3/4).
    #[tool(
        name = "memory_changelog",
        description = "Read the tail of CHANGELOG.md (or automation.changelog_path). Same repo as JOICY_REPO_ROOT / cwd."
    )]
    pub async fn memory_changelog(
        &self,
        params: Parameters<MemoryChangelogParams>,
    ) -> Result<String, String> {
        let n = params.0.lines;
        tokio::task::spawn_blocking(move || {
            let root = workspace::resolve_repo_root().map_err(|e| e.to_string())?;
            let cfg = workspace::load_repo_config(&root).map_err(|e| e.to_string())?;
            crate::automation::read_changelog_tail(&root, &cfg, n).map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| format!("join error: {e}"))?
    }

    /// Write a normal Obsidian-ready note into the configured vault (`export_root` / `JOICY_VAULT_ROOT`).
    #[tool(
        name = "memory_vault_note",
        description = "Create a markdown file under vault/{namespace}/{subfolder}/ (default notes/) with Joicy front matter. Set `file_stem` (e.g. My-Topic) for stable names so wikilinks [[My-Topic]] and the graph work. Repo: JOICY_REPO_ROOT or cwd."
    )]
    pub async fn memory_vault_note(
        &self,
        params: Parameters<MemoryVaultNoteParams>,
    ) -> Result<String, String> {
        let p = params.0;
        let content_in = p.content;
        let title = p.title;
        let subfolder = p.subfolder;
        let language = p.language;
        let index_memory = p.index_memory;
        let file_stem = p.file_stem;
        tokio::task::spawn_blocking(move || {
            let root = workspace::resolve_repo_root().map_err(|e| e.to_string())?;
            let cfg = workspace::load_repo_config(&root).map_err(|e| e.to_string())?;

            let sub = subfolder
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("notes");
            let sub = sub.trim().trim_matches('/');

            let (logical_label, body) = match title.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                Some(t) => {
                    let logical = format!("{sub}/{t}");
                    let body = if content_in.trim_start().starts_with('#') {
                        content_in.clone()
                    } else {
                        format!("# {t}\n\n{content_in}")
                    };
                    (logical, body)
                }
                None => (format!("{sub}/agent-note"), content_in.clone()),
            };

            let path = crate::vault_markdown::write_vault_markdown_file(
                &root,
                &cfg,
                crate::vault_markdown::VaultMarkdownWrite {
                    logical_label: &logical_label,
                    body: &body,
                    language: &language,
                    subfolder: Some(sub),
                    source: "mcp",
                    file_stem: file_stem.as_deref(),
                },
            )
            .map_err(|e| e.to_string())?;

            if index_memory {
                let mut bank = workspace::open_bank(&root, &cfg).map_err(|e| e.to_string())?;
                let ctx = CodeContext {
                    content: body.clone(),
                    file_path: logical_label.clone(),
                    language: language.clone(),
                    metadata: Vec::new(),
                    timestamp: timestamp(),
                };
                bank.store(ctx).map_err(|e| e.to_string())?;
            }

            Ok(format!(
                "Wrote vault note: {}\n(joicy_label: {}; indexed_memory: {index_memory})",
                path.display(),
                logical_label
            ))
        })
        .await
        .map_err(|e| format!("join error: {e}"))?
    }
}

fn format_results(hits: &[CodeContext]) -> String {
    if hits.is_empty() {
        return "No results.".to_string();
    }
    let mut out = String::new();
    for (i, h) in hits.iter().enumerate() {
        out.push_str(&format!(
            "---\n{} | {} | {}\n{}\n",
            i + 1,
            h.file_path,
            h.language,
            truncate_content(&h.content, 800)
        ));
    }
    out
}

fn truncate_content(t: &str, max_chars: usize) -> String {
    let count = t.chars().count();
    if count <= max_chars {
        return t.to_string();
    }
    format!("{}…", t.chars().take(max_chars).collect::<String>())
}

/// Run MCP over stdin/stdout until the client disconnects. **Do not print to stdout** (protocol).
pub fn serve_stdio() -> crate::error::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| crate::error::Error::Mcp(format!("tokio: {e}")))?;
    rt.block_on(async {
        let server = JoicyMcpServer::new();
        let transport = transport::stdio();
        let running = serve_server(server, transport)
            .await
            .map_err(|e| crate::error::Error::Mcp(e.to_string()))?;
        let _ = running.waiting().await;
        Ok::<(), crate::error::Error>(())
    })?;
    Ok(())
}
