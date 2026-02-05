use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum GrovError {
    #[error("{0} is not a bare repository")]
    NotBareRepo(PathBuf),

    #[error("could not find a bare repository from {0}")]
    BareRepoNotFound(PathBuf),

    #[error("worktree already exists at {0}")]
    WorktreeAlreadyExists(PathBuf),

    #[error("worktree not found: {0}")]
    WorktreeNotFound(String),

    #[error("worktree has uncommitted changes (use --force to override)")]
    WorktreeDirty,

    #[error("branch not found: {0}")]
    BranchNotFound(String),

    #[error("invalid branch name: {0}")]
    InvalidBranchName(String),

    #[error("git command failed: {0}")]
    GitCommandFailed(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, GrovError>;
