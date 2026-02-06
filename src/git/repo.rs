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
/// 2. Check if `start` contains a `repo.git` child that is a bare repo.
/// 3. Use `git rev-parse --git-common-dir` (works inside worktrees).
/// 4. Walk up parent directories, checking each for `repo.git` child or bare repo.
pub fn find_bare_repo(start: &Path) -> Result<PathBuf> {
    let start =
        std::fs::canonicalize(start).map_err(|_| GrovError::BareRepoNotFound(start.into()))?;

    // 1. Direct check
    if is_bare_repo(&start) {
        return Ok(start);
    }

    // 2. Check for repo.git child
    let repo_git = start.join("repo.git");
    if repo_git.is_dir() && is_bare_repo(&repo_git) {
        return Ok(repo_git);
    }

    // 3. Try git rev-parse --git-common-dir from within a worktree
    if let Ok(output) = run_git(
        None,
        &[
            "-C",
            &start.to_string_lossy(),
            "rev-parse",
            "--git-common-dir",
        ],
    ) && output.status.success()
        && !output.stdout.is_empty()
    {
        let common_dir = PathBuf::from(&output.stdout);
        let common_dir = if common_dir.is_absolute() {
            common_dir
        } else {
            start.join(&common_dir)
        };
        if let Ok(canonical) = std::fs::canonicalize(&common_dir)
            && is_bare_repo(&canonical)
        {
            return Ok(canonical);
        }
    }

    // 4. Walk up parent directories, checking for repo.git child or bare repo
    let mut current = start.clone();
    loop {
        if !current.pop() {
            break;
        }

        let repo_git = current.join("repo.git");
        if repo_git.is_dir() && is_bare_repo(&repo_git) {
            return Ok(repo_git);
        }

        if is_bare_repo(&current) {
            return Ok(current);
        }
    }

    Err(GrovError::BareRepoNotFound(start))
}

/// Get the default branch by parsing `refs/remotes/origin/HEAD`,
/// falling back to the bare repo's own HEAD if the remote ref isn't set
/// (common with local clones).
pub fn default_branch(repo: &Path) -> Result<String> {
    if let Ok(output) = run_git_ok(Some(repo), &["symbolic-ref", "refs/remotes/origin/HEAD"]) {
        let branch = output
            .strip_prefix("refs/remotes/origin/")
            .unwrap_or(&output);
        return Ok(branch.to_string());
    }

    // Fall back to the bare repo's own HEAD
    let output = run_git_ok(Some(repo), &["symbolic-ref", "HEAD"])?;
    let branch = output.strip_prefix("refs/heads/").unwrap_or(&output);
    Ok(branch.to_string())
}
