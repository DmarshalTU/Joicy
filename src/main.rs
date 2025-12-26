//! Joicy CLI entry point

#[cfg(feature = "cli")]
use joicy::cli::{Cli, Commands};
use joicy::error::Result;

fn main() -> Result<()> {
    #[cfg(feature = "cli")]
    {
        use joicy::cli::{clean, export, init, search, status, sync};
        
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
            Commands::Sync { force } => sync(force)?,
            Commands::Status => status()?,
            Commands::Clean { days } => clean(days)?,
            Commands::Export { output } => export(output.as_deref())?,
        }
    }

    #[cfg(not(feature = "cli"))]
    {
        eprintln!("CLI feature is not enabled. Please enable it in Cargo.toml");
        std::process::exit(1);
    }

    Ok(())
}
