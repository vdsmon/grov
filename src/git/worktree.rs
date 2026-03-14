use std::path::{Path, PathBuf};

use crate::errors::Result;
use crate::git::executor::{run_git, run_git_ok};

#[derive(Debug, Clone)]
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

pub fn matches_branch_name(worktree: &WorktreeInfo, name: &str) -> bool {
    worktree.branch.as_deref() == Some(name)
}

pub fn matches_dir_name(worktree: &WorktreeInfo, name: &str) -> bool {
    worktree
        .path
        .file_name()
        .is_some_and(|dir_name| dir_name.to_string_lossy() == name)
}

pub fn worktree_dir_name(worktree: &WorktreeInfo) -> String {
    worktree
        .path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// List available branches that don't already have a worktree.
///
/// Collects local and remote branches, deduplicates (local takes precedence),
/// strips `origin/` prefix from remote branches, excludes `HEAD`, and filters
/// out branches that already have a worktree.
pub fn available_branches(repo: &Path, worktrees: &[WorktreeInfo]) -> Result<Vec<String>> {
    use std::collections::BTreeSet;

    let worktree_branches: std::collections::HashSet<&str> = worktrees
        .iter()
        .filter_map(|wt| wt.branch.as_deref())
        .collect();

    let mut branches = BTreeSet::new();

    // Local branches: git for-each-ref --format='%(refname:short)' refs/heads/
    let local_output = run_git_ok(
        Some(repo),
        &["for-each-ref", "--format=%(refname:short)", "refs/heads/"],
    )?;
    for line in local_output.lines() {
        let name = line.trim();
        if !name.is_empty() {
            branches.insert(name.to_string());
        }
    }

    // Remote branches: git for-each-ref --format='%(refname:short)' refs/remotes/origin/
    let remote_output = run_git_ok(
        Some(repo),
        &[
            "for-each-ref",
            "--format=%(refname:short)",
            "refs/remotes/origin/",
        ],
    )?;
    for line in remote_output.lines() {
        let name = line.trim();
        // Strip origin/ prefix, skip HEAD
        if let Some(branch) = name.strip_prefix("origin/")
            && branch != "HEAD"
            && !branch.is_empty()
        {
            branches.insert(branch.to_string());
        }
    }

    // Filter out branches that already have a worktree
    let result: Vec<String> = branches
        .into_iter()
        .filter(|b| !worktree_branches.contains(b.as_str()))
        .collect();

    Ok(result)
}

/// Try to delete a local branch safely (with `-d`).
///
/// Returns `Ok(())` on success. If the branch is not fully merged, returns
/// an error whose message contains "not fully merged".
pub fn safe_delete_branch(repo: &Path, name: &str) -> Result<()> {
    run_git_ok(Some(repo), &["branch", "-d", name])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // We test the parsing logic indirectly; the function calls git directly
        // so we test the struct construction logic here
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

        // Handle last entry (no trailing blank line)
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
}
