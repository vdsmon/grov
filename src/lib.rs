pub mod cli;
pub mod commands;
pub mod errors;
pub mod git;
pub mod paths;

use anyhow::Context;
use clap::Parser;

use cli::{Cli, Commands};

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { url, name, path } => {
            commands::init::execute(&url, name.as_deref(), path.as_deref())
                .context("init failed")?;
        }
        Commands::Add { branch, base, path } => {
            commands::add::execute(&branch, base.as_deref(), path.as_deref())
                .context("add failed")?;
        }
        Commands::List { compact } => {
            commands::list::execute(compact).context("list failed")?;
        }
        Commands::Remove {
            name,
            delete_branch,
            force,
        } => {
            commands::remove::execute(&name, delete_branch, force).context("remove failed")?;
        }
        Commands::Completions { shell } => {
            commands::completions::execute(shell);
        }
    }

    Ok(())
}
