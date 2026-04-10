//! Joicy CLI entry point

#[cfg(feature = "cli")]
use joicy::cli::{Cli, Commands};
use joicy::error::Result;

fn main() -> Result<()> {
    #[cfg(feature = "cli")]
    {
        use joicy::cli::{
            add, automation_on_commit, changelog_show, clean, export, hooks_install, init, search,
            status, sync, vault_export,
        };
        use joicy::cli::{
            AutomationCommands, ChangelogCommands, HooksCommands, McpCommands, VaultCommands,
        };
        
        let cli = Cli::parse();
        
        if cli.verbose {
            #[cfg(feature = "dev")]
            {
                env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
                    .init();
            }
        }

        match cli.command {
            Commands::Init { path } => init(&path)?,
            Commands::Search { query, file, limit } => {
                search(&query, file.as_deref(), limit)?
            }
            Commands::Add {
                text,
                file,
                label,
                language,
            } => add(text, file, label, language)?,
            Commands::Sync { force } => sync(force)?,
            Commands::Status => status()?,
            Commands::Clean { days } => clean(days)?,
            Commands::Export { output } => export(output.as_deref())?,
            Commands::Changelog { sub } => match sub {
                ChangelogCommands::Show { lines } => changelog_show(lines)?,
            },
            Commands::Vault { sub } => match sub {
                VaultCommands::Export {
                    dir,
                    namespace,
                    limit,
                } => vault_export(dir.as_deref(), namespace.as_deref(), limit)?,
            },
            Commands::Hooks { sub } => match sub {
                HooksCommands::Install => hooks_install()?,
            },
            Commands::Automation { sub } => match sub {
                AutomationCommands::OnCommit => automation_on_commit()?,
            },
            Commands::Mcp { sub } => match sub {
                McpCommands::Serve => joicy::mcp::serve_stdio()?,
            },
        }
    }

    #[cfg(not(feature = "cli"))]
    {
        eprintln!("CLI feature is not enabled. Please enable it in Cargo.toml");
        std::process::exit(1);
    }

    Ok(())
}
