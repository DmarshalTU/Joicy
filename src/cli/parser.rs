//! CLI argument parsing

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Joicy - Team Memory Bank System
#[derive(Parser, Debug)]
#[command(name = "joicy")]
#[command(about = "Team memory bank system for AI-assisted development")]
#[command(version = env!("JOICY_CLI_VERSION"))]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<String>,

    /// Command to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize memory bank
    Init {
        /// Repository path
        #[arg(default_value = ".")]
        path: String,
    },

    /// Search memory bank (full-text; local POC)
    Search {
        /// Search query (FTS; empty lists recent entries)
        query: String,

        /// Filter by file path substring
        #[arg(short, long)]
        file: Option<String>,

        /// Limit number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Add an entry to the local memory bank
    Add {
        /// Store this literal text
        #[arg(long)]
        text: Option<String>,

        /// Read content from this file
        #[arg(long)]
        file: Option<PathBuf>,

        /// Logical path label (default: `snippet` or file name)
        #[arg(long)]
        label: Option<String>,

        /// Language tag (e.g. rust, md)
        #[arg(short, long, default_value = "text")]
        language: String,
    },

    /// Sync with central memory bank
    Sync {
        /// Force full sync
        #[arg(short, long)]
        force: bool,
    },

    /// Show memory bank status
    Status,

    /// Clean old entries
    Clean {
        /// Days to keep
        #[arg(short, long, default_value = "30")]
        days: u64,
    },

    /// Export memory bank
    Export {
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Changelog appended on each commit (post-commit step 3/4)
    Changelog {
        /// Changelog subcommand
        #[command(subcommand)]
        sub: ChangelogCommands,
    },

    /// Markdown vault for Obsidian (human-readable layer; dimension 2)
    Vault {
        /// Vault operation
        #[command(subcommand)]
        sub: VaultCommands,
    },

    /// Install git hooks for automatic capture after each commit
    Hooks {
        /// Hook subcommand
        #[command(subcommand)]
        sub: HooksCommands,
    },

    /// Run automation steps (normally called from a git hook)
    Automation {
        /// Automation subcommand
        #[command(subcommand)]
        sub: AutomationCommands,
    },

    /// Model Context Protocol (IDE / agent integration)
    Mcp {
        /// MCP subcommand
        #[command(subcommand)]
        sub: McpCommands,
    },
}

/// Changelog (Joicy-managed `CHANGELOG.md` by default)
#[derive(Subcommand, Debug)]
pub enum ChangelogCommands {
    /// Print the last N lines of the changelog file
    Show {
        /// Number of lines from the end of the file
        #[arg(short = 'n', long, default_value_t = 80)]
        lines: usize,
    },
}

/// MCP subcommands
#[derive(Subcommand, Debug)]
pub enum McpCommands {
    /// MCP over stdin/stdout (add to Cursor: command `joicy`, args `mcp serve`, cwd = repo)
    Serve,
}

/// Git hook installation
#[derive(Subcommand, Debug)]
pub enum HooksCommands {
    /// Install `post-commit` to run Joicy automation
    Install,
}

/// Automation invoked by hooks (or manually)
#[derive(Subcommand, Debug)]
pub enum AutomationCommands {
    /// After a commit: store HEAD in the DB and export the vault if enabled
    OnCommit,
}

/// Vault subcommands
#[derive(Subcommand, Debug)]
pub enum VaultCommands {
    /// Export local DB entries as Markdown notes (Zettelkasten-friendly front matter)
    Export {
        /// Vault root directory [default: .joicy/vault in this repo]
        #[arg(short = 'o', long)]
        dir: Option<PathBuf>,

        /// Subfolder for this repo (default: repository folder name). Use the same `--dir` and a different `--namespace` per repo to build one shared Obsidian vault.
        #[arg(short = 'n', long)]
        namespace: Option<String>,

        /// Max entries to export (newest first)
        #[arg(short = 'l', long, default_value_t = 10_000)]
        limit: usize,
    },
}

impl Cli {
    /// Parse CLI arguments from environment
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}

