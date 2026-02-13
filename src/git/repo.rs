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

/// Get the default branch by parsing `refs/remotes/origin/HEAD`.
pub fn default_branch(repo: &Path) -> Result<String> {
    let output = run_git_ok(Some(repo), &["symbolic-ref", "refs/remotes/origin/HEAD"])?;

    // Output is like "refs/remotes/origin/main"
    let branch = output
        .strip_prefix("refs/remotes/origin/")
        .unwrap_or(&output);

    Ok(branch.to_string())
}

/// Detect the current branch of the worktree at `cwd`.
///
/// Returns `Ok(None)` when:
/// - `cwd` is not inside a git work tree (e.g. bare repo, non-repo dir)
/// - HEAD is detached
/// - git commands fail (non-zero exit)
///
/// IO/spawn errors propagate via `?`.
pub fn current_branch(cwd: &Path) -> Result<Option<String>> {
    let cwd_lossy = cwd.to_string_lossy();

    let inside = run_git(
        None,
        &["-C", &cwd_lossy, "rev-parse", "--is-inside-work-tree"],
    )?;
    if !inside.status.success() || inside.stdout != "true" {
        return Ok(None);
    }

    let head = run_git(
        None,
        &["-C", &cwd_lossy, "rev-parse", "--abbrev-ref", "HEAD"],
    )?;
    if !head.status.success() || head.stdout == "HEAD" {
        return Ok(None);
    }

    Ok(Some(head.stdout))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    fn git(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("failed to run git")
            .status;
        assert!(status.success(), "git {args:?} failed");
    }

    fn init_repo(dir: &Path) {
        git(dir, &["init", "-b", "main"]);
        git(dir, &["config", "user.email", "test@test.com"]);
        git(dir, &["config", "user.name", "Test"]);
    }

    #[test]
    fn test_current_branch_normal_worktree() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        init_repo(dir);
        git(dir, &["commit", "--allow-empty", "-m", "init"]);

        let branch = current_branch(dir).unwrap();
        assert_eq!(branch, Some("main".to_string()));
    }

    #[test]
    fn test_current_branch_bare_repo_returns_none() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        git(dir, &["init", "--bare"]);

        let branch = current_branch(dir).unwrap();
        assert_eq!(branch, None);
    }

    #[test]
    fn test_current_branch_detached_head_returns_none() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        init_repo(dir);
        git(dir, &["commit", "--allow-empty", "-m", "init"]);
        git(dir, &["checkout", "--detach"]);

        let branch = current_branch(dir).unwrap();
        assert_eq!(branch, None);
    }

    #[test]
    fn test_current_branch_nonexistent_dir_returns_none() {
        let dir = Path::new("/tmp/grov_nonexistent_test_dir_12345");
        let branch = current_branch(dir).unwrap();
        assert_eq!(branch, None);
    }
}
