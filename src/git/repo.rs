use std::path::{Path, PathBuf};

use crate::errors::{GrovError, Result};
use crate::git::executor::{run_git, run_git_ok};

/// Check whether the given path is a bare git repository.
pub fn is_bare_repo(path: &Path) -> bool {
    let result = run_git(Some(path), &["rev-parse", "--is-bare-repository"]);
    matches!(result, Ok(output) if output.stdout == "true")
}

/// Find the bare repository starting from `start`.
///
/// Strategy:
/// 1. If `start` is itself a bare repo, return it.
/// 2. Use `git rev-parse --git-common-dir` (works inside worktrees).
/// 3. Walk up parent directories looking for a bare repo.
pub fn find_bare_repo(start: &Path) -> Result<PathBuf> {
    let start =
        std::fs::canonicalize(start).map_err(|_| GrovError::BareRepoNotFound(start.into()))?;

    // 1. Direct check
    if is_bare_repo(&start) {
        return Ok(start);
    }

    // 2. Try git rev-parse --git-common-dir from within a worktree
    if let Ok(output) = run_git(
        None,
        &[
            "-C",
            &start.to_string_lossy(),
            "rev-parse",
            "--git-common-dir",
        ],
    ) {
        if output.status.success() && !output.stdout.is_empty() {
            let common_dir = PathBuf::from(&output.stdout);
            let common_dir = if common_dir.is_absolute() {
                common_dir
            } else {
                start.join(&common_dir)
            };
            if let Ok(canonical) = std::fs::canonicalize(&common_dir) {
                if is_bare_repo(&canonical) {
                    return Ok(canonical);
                }
            }
        }
    }

    // 3. Walk up parent directories
    let mut current = start.clone();
    loop {
        if is_bare_repo(&current) {
            return Ok(current);
        }
        if !current.pop() {
            break;
        }
    }

    Err(GrovError::BareRepoNotFound(start))
}

/// Get the default branch by parsing `refs/remotes/origin/HEAD`.
pub fn default_branch(repo: &Path) -> Result<String> {
    let output = run_git_ok(Some(repo), &["symbolic-ref", "refs/remotes/origin/HEAD"])?;

    // Output is like "refs/remotes/origin/main"
    let branch = output
        .strip_prefix("refs/remotes/origin/")
        .unwrap_or(&output);

    Ok(branch.to_string())
}
