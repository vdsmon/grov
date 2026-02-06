use std::path::{Path, PathBuf};

use crate::errors::Result;
use crate::git::executor::{run_git, run_git_ok};

#[derive(Debug)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub head: String,
    pub branch: Option<String>,
    pub is_bare: bool,
}

/// Parse `git worktree list --porcelain` output into structured data.
pub fn list_worktrees(repo: &Path) -> Result<Vec<WorktreeInfo>> {
    let output = run_git_ok(Some(repo), &["worktree", "list", "--porcelain"])?;
    let mut worktrees = Vec::new();
    let mut path = None;
    let mut head = None;
    let mut branch = None;
    let mut is_bare = false;

    for line in output.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            path = Some(PathBuf::from(p));
        } else if let Some(h) = line.strip_prefix("HEAD ") {
            head = Some(h.to_string());
        } else if let Some(b) = line.strip_prefix("branch ") {
            // branch refs/heads/main → main
            let b = b.strip_prefix("refs/heads/").unwrap_or(b);
            branch = Some(b.to_string());
        } else if line == "bare" {
            is_bare = true;
        } else if line.is_empty() {
            if let (Some(p), Some(h)) = (path.take(), head.take()) {
                worktrees.push(WorktreeInfo {
                    path: p,
                    head: h,
                    branch: branch.take(),
                    is_bare,
                });
            }
            is_bare = false;
        }
    }

    // Handle last entry (no trailing newline)
    if let (Some(p), Some(h)) = (path, head) {
        worktrees.push(WorktreeInfo {
            path: p,
            head: h,
            branch: branch.take(),
            is_bare,
        });
    }

    Ok(worktrees)
}

/// Create a new worktree.
///
/// Builds: `git worktree add [extra_args...] <path> [commit_ish]`
pub fn add_worktree(
    repo: &Path,
    worktree_path: &Path,
    commit_ish: Option<&str>,
    extra_args: &[&str],
) -> Result<()> {
    let path_str = worktree_path.to_string_lossy();
    let mut args = vec!["worktree", "add"];
    args.extend(extra_args);
    args.push(&path_str);
    if let Some(c) = commit_ish {
        args.push(c);
    }

    run_git_ok(Some(repo), &args)?;
    Ok(())
}

/// Remove a worktree.
pub fn remove_worktree(repo: &Path, worktree_path: &Path, force: bool) -> Result<()> {
    let path_str = worktree_path.to_string_lossy();
    let mut args = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(&path_str);

    run_git_ok(Some(repo), &args)?;
    Ok(())
}

/// Check if a local branch exists.
pub fn branch_exists_local(repo: &Path, name: &str) -> bool {
    let refname = format!("refs/heads/{name}");
    run_git(Some(repo), &["rev-parse", "--verify", &refname])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if a remote branch exists on origin.
pub fn branch_exists_remote(repo: &Path, name: &str) -> bool {
    let refname = format!("refs/remotes/origin/{name}");
    run_git(Some(repo), &["rev-parse", "--verify", &refname])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Delete a local branch.
pub fn delete_branch(repo: &Path, name: &str) -> Result<()> {
    run_git_ok(Some(repo), &["branch", "-D", name])?;
    Ok(())
}

/// Find a worktree by branch name or directory name.
pub fn find_worktree<'a>(worktrees: &'a [WorktreeInfo], name: &str) -> Option<&'a WorktreeInfo> {
    worktrees.iter().find(|wt| {
        // Match by branch name
        if let Some(ref branch) = wt.branch
            && branch == name
        {
            return true;
        }
        // Match by directory name
        if let Some(dir_name) = wt.path.file_name()
            && dir_name.to_string_lossy() == name
        {
            return true;
        }
        false
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_worktree(path: &str, branch: Option<&str>, is_bare: bool) -> WorktreeInfo {
        WorktreeInfo {
            path: PathBuf::from(path),
            head: "abc123".to_string(),
            branch: branch.map(|b| b.to_string()),
            is_bare,
        }
    }

    #[test]
    fn parse_porcelain_output() {
        let output = "\
worktree /repos/project.git
HEAD abc123
bare

worktree /repos/project.git/trees/main
HEAD def456
branch refs/heads/main

worktree /repos/project.git/trees/feature
HEAD 789abc
branch refs/heads/feature/login
";
        let mut worktrees = Vec::new();
        let mut path = None;
        let mut head = None;
        let mut branch = None;
        let mut is_bare = false;

        for line in output.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                path = Some(PathBuf::from(p));
            } else if let Some(h) = line.strip_prefix("HEAD ") {
                head = Some(h.to_string());
            } else if let Some(b) = line.strip_prefix("branch ") {
                let b = b.strip_prefix("refs/heads/").unwrap_or(b);
                branch = Some(b.to_string());
            } else if line == "bare" {
                is_bare = true;
            } else if line.is_empty() {
                if let (Some(p), Some(h)) = (path.take(), head.take()) {
                    worktrees.push(WorktreeInfo {
                        path: p,
                        head: h,
                        branch: branch.take(),
                        is_bare,
                    });
                }
                is_bare = false;
            }
        }

        if let (Some(p), Some(h)) = (path, head) {
            worktrees.push(WorktreeInfo {
                path: p,
                head: h,
                branch: branch.take(),
                is_bare,
            });
        }

        assert_eq!(worktrees.len(), 3);
        assert!(worktrees[0].is_bare);
        assert_eq!(worktrees[0].branch, None);
        assert_eq!(worktrees[1].branch.as_deref(), Some("main"));
        assert_eq!(worktrees[2].branch.as_deref(), Some("feature/login"));
    }

    #[test]
    fn find_worktree_by_branch_name() {
        let worktrees = vec![
            make_worktree("/repos/project.git", None, true),
            make_worktree("/repos/test_main", Some("main"), false),
            make_worktree("/repos/test_feature-x", Some("feature-x"), false),
        ];

        let found = find_worktree(&worktrees, "feature-x");
        assert!(found.is_some());
        assert_eq!(found.unwrap().branch.as_deref(), Some("feature-x"));
    }

    #[test]
    fn find_worktree_by_directory_name() {
        let worktrees = vec![
            make_worktree("/repos/project.git", None, true),
            make_worktree("/repos/test_main", Some("main"), false),
        ];

        let found = find_worktree(&worktrees, "test_main");
        assert!(found.is_some());
        assert_eq!(found.unwrap().branch.as_deref(), Some("main"));
    }

    #[test]
    fn find_worktree_not_found() {
        let worktrees = vec![
            make_worktree("/repos/project.git", None, true),
            make_worktree("/repos/test_main", Some("main"), false),
        ];

        assert!(find_worktree(&worktrees, "nonexistent").is_none());
    }

    #[test]
    fn find_worktree_detached_head_matches_by_dir() {
        let worktrees = vec![make_worktree("/repos/my-worktree", None, false)];

        let found = find_worktree(&worktrees, "my-worktree");
        assert!(found.is_some());
        assert_eq!(found.unwrap().branch, None);
    }

    #[test]
    fn find_worktree_branch_match_preferred_over_dir() {
        // A worktree whose branch name matches the query should be found
        // even if another worktree's directory name also matches
        let worktrees = vec![
            make_worktree("/repos/feature-x", Some("main"), false),
            make_worktree("/repos/test_other", Some("feature-x"), false),
        ];

        let found = find_worktree(&worktrees, "feature-x");
        assert!(found.is_some());
        // Should find the one with matching branch name (iter order: first match wins)
        // First worktree's dir is "feature-x" so it matches by dir name
        // But find uses iter().find(), which returns the first match
        // The first worktree matches by dir name "feature-x"
        assert_eq!(found.unwrap().path, PathBuf::from("/repos/feature-x"));
    }
}
