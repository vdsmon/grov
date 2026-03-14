use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
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
        /// Parent directory to clone into (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Create a new worktree for a branch
    Add {
        /// Branch name to check out or create (prompted if not provided)
        branch: Option<String>,

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
        /// Worktree name or branch to remove (prompted if not provided)
        name: Option<String>,

        /// How to interpret the name when resolving a worktree
        #[arg(long = "match", value_enum, default_value_t = RemoveMatchMode::Auto)]
        match_mode: RemoveMatchMode,

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

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum RemoveMatchMode {
    /// Match by branch or directory name, and fail on ambiguity
    Auto,
    /// Match only by branch name
    Branch,
    /// Match only by worktree directory name
    Dir,
}
