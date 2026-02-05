use std::path::Path;

use crate::git::executor::run_git_ok;
use crate::git::repo::default_branch;
use crate::git::worktree::add_worktree;
use crate::paths::{repo_name_from_url, worktree_dir};

pub fn execute(url: &str, name: Option<&str>, path: Option<&Path>) -> anyhow::Result<()> {
    let repo_name = name
        .map(String::from)
        .unwrap_or_else(|| repo_name_from_url(url));
    let bare_dir_name = format!("{repo_name}.git");

    let parent = match path {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir()?,
    };

    let bare_path = parent.join(&bare_dir_name);

    if bare_path.exists() {
        anyhow::bail!("directory already exists: {}", bare_path.display());
    }

    // Clone bare
    let bare_str = bare_path.to_string_lossy().to_string();
    run_git_ok(None, &["clone", "--bare", url, &bare_str])?;

    // Fix fetch refspec so `git fetch` works properly
    run_git_ok(
        Some(&bare_path),
        &[
            "config",
            "remote.origin.fetch",
            "+refs/heads/*:refs/remotes/origin/*",
        ],
    )?;

    // Fetch to populate remote tracking branches
    run_git_ok(Some(&bare_path), &["fetch", "origin"])?;

    // Detect default branch
    let branch = default_branch(&bare_path)?;

    // Create initial worktree
    let wt_path = worktree_dir(&bare_path, &branch);
    if let Some(parent) = wt_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    add_worktree(&bare_path, &wt_path, Some(&branch), &[])?;

    println!("Initialized bare repo at {bare_dir_name}, worktree at trees/{branch}");
    Ok(())
}
