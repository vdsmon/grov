use crate::git::repo::find_bare_repo;
use crate::git::status::is_dirty;
use crate::git::worktree::{delete_branch, find_worktree, list_worktrees, remove_worktree};

pub fn execute(name: &str, do_delete_branch: bool, force: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let repo = find_bare_repo(&cwd)?;
    let worktrees = list_worktrees(&repo)?;

    let wt = find_worktree(&worktrees, name)
        .ok_or_else(|| anyhow::anyhow!("worktree not found: {name}"))?;

    if wt.is_bare {
        anyhow::bail!("cannot remove the bare repository entry");
    }

    // Check for dirty state
    if !force && is_dirty(&wt.path).unwrap_or(false) {
        anyhow::bail!("worktree has uncommitted changes (use --force to override)");
    }

    let branch_name = wt.branch.clone();
    let wt_path = wt.path.clone();

    remove_worktree(&repo, &wt_path, force)?;

    println!("Removed worktree at {}", wt_path.display());

    if do_delete_branch {
        if let Some(ref branch) = branch_name {
            delete_branch(&repo, branch)?;
            println!("Deleted branch {branch}");
        }
    }

    Ok(())
}
