//! CLI argument parsing

use clap::{Parser, Subcommand};

/// Joicy - Team Memory Bank System
#[derive(Parser, Debug)]
#[command(name = "joicy")]
#[command(about = "Team memory bank system for AI-assisted development")]
#[command(version)]
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

    /// Search memory bank
    Search {
        /// Search query
        query: String,

        /// Filter by file path
        #[arg(short, long)]
        file: Option<String>,

        /// Limit number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
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
}

impl Cli {
    /// Parse CLI arguments from environment
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}

