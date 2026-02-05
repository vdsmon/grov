use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_complete::Shell;

/// An opinionated bare-repo-only git worktree manager
#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Clone a repo as bare and create an initial worktree
    Init {
        /// Repository URL to clone (prompted if not provided)
        #[arg(long)]
        url: Option<String>,

        /// Project directory name (defaults to repo name from URL)
        #[arg(long)]
        name: Option<String>,

        /// Worktree prefix (e.g. "dp" creates dp_main, dp_feature-x)
        #[arg(long)]
        prefix: Option<String>,

        /// Parent directory to clone into (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Create a new worktree for a branch
    Add {
        /// Branch name to check out or create
        branch: String,

        /// Base branch for new branches (defaults to the default branch)
        #[arg(long)]
        base: Option<String>,

        /// Custom path for the worktree
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// List all worktrees
    #[command(alias = "ls")]
    List {
        /// Show only branch names, one per line
        #[arg(long)]
        compact: bool,
    },

    /// Remove a worktree
    #[command(alias = "rm")]
    Remove {
        /// Worktree name or branch to remove
        name: String,

        /// Also delete the local branch
        #[arg(long)]
        delete_branch: bool,

        /// Force removal even if worktree has uncommitted changes
        #[arg(long)]
        force: bool,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}
