pub mod cli;
pub mod commands;
pub mod config;
pub mod errors;
pub mod git;
pub mod paths;
pub mod tui;

use anyhow::Context;
use clap::Parser;

use cli::{Cli, Commands};

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            commands::init::execute(path.as_deref()).context("init failed")?;
        }
        Commands::Add { branch, base, path } => {
            commands::add::execute(branch.as_deref(), base.as_deref(), path.as_deref())
                .context("add failed")?;
        }
        Commands::List { compact } => {
            commands::list::execute(compact).context("list failed")?;
        }
        Commands::Remove {
            name,
            match_mode,
            delete_branch,
            force,
        } => {
            commands::remove::execute(name.as_deref(), match_mode, delete_branch, force)
                .context("remove failed")?;
        }
        Commands::Completions { shell } => {
            commands::completions::execute(shell);
        }
    }

    Ok(())
}
