use std::path::Path;

use crate::errors::Result;
use crate::git::executor::run_git_ok;

/// Check if a worktree has uncommitted changes.
pub fn is_dirty(worktree_path: &Path) -> Result<bool> {
    let path_str = worktree_path.to_string_lossy();
    let output = run_git_ok(None, &["-C", &path_str, "status", "--porcelain"])?;
    Ok(!output.is_empty())
}

/// Get ahead/behind counts relative to upstream.
/// Returns `None` if no upstream is configured.
pub fn ahead_behind(worktree_path: &Path) -> Result<Option<(u32, u32)>> {
    let path_str = worktree_path.to_string_lossy();
    let output = run_git_ok(
        None,
        &[
            "-C",
            &path_str,
            "rev-list",
            "--left-right",
            "--count",
            "HEAD...@{upstream}",
        ],
    );

    match output {
        Ok(text) => {
            let parts: Vec<&str> = text.split('\t').collect();
            if parts.len() == 2 {
                let ahead = parts[0].parse().unwrap_or(0);
                let behind = parts[1].parse().unwrap_or(0);
                Ok(Some((ahead, behind)))
            } else {
                Ok(None)
            }
        }
        Err(_) => Ok(None), // No upstream configured
    }
}
