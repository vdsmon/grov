use std::path::Path;

use console::style;

use crate::config::read_config;
use crate::git::executor::run_git_ok;
use crate::git::repo::{default_branch, find_bare_repo};
use crate::git::worktree::{add_worktree, branch_exists_local, branch_exists_remote};
use crate::paths::worktree_dir;

pub fn execute(branch: &str, base: Option<&str>, custom_path: Option<&Path>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let repo = find_bare_repo(&cwd)?;
    let config = read_config(&repo);

    // Fetch latest
    let _ = run_git_ok(Some(&repo), &["fetch", "origin"]);

    // Determine worktree path
    let wt_path = match custom_path {
        Some(p) => p.to_path_buf(),
        None => worktree_dir(&repo, branch, &config.worktree.prefix),
    };

    // Check if worktree dir already exists
    if wt_path.exists() {
        anyhow::bail!("worktree directory already exists at {}", wt_path.display());
    }

    let remote_ref = format!("origin/{branch}");

    if branch_exists_local(&repo, branch) {
        // Local branch exists → check it out
        add_worktree(&repo, &wt_path, Some(branch), &[])?;
    } else if branch_exists_remote(&repo, branch) {
        // Remote branch exists → git worktree add --track -b <branch> <path> origin/<branch>
        add_worktree(
            &repo,
            &wt_path,
            Some(&remote_ref),
            &["--track", "-b", branch],
        )?;
    } else {
        // New branch from base
        let base_branch = match base {
            Some(b) => b.to_string(),
            None => default_branch(&repo)?,
        };
        add_worktree(&repo, &wt_path, Some(&base_branch), &["-b", branch])?;
    }

    println!(
        "{} Created worktree at {} on branch {}",
        style("✓").green().bold(),
        style(wt_path.display()).bold(),
        style(branch).cyan().bold(),
    );
    Ok(())
}
