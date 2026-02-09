use console::style;

use crate::cli::RemoveMatchMode;
use crate::git::repo::find_bare_repo;
use crate::git::status::is_dirty;
use crate::git::worktree::{
    WorktreeInfo, delete_branch, list_worktrees, matches_branch_name, matches_dir_name,
    remove_worktree, worktree_dir_name,
};

pub fn execute(
    name: &str,
    match_mode: RemoveMatchMode,
    do_delete_branch: bool,
    force: bool,
) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let repo = find_bare_repo(&cwd)?;
    let worktrees = list_worktrees(&repo)?;

    let matches: Vec<&WorktreeInfo> = worktrees
        .iter()
        .filter(|worktree| match match_mode {
            RemoveMatchMode::Auto => {
                matches_branch_name(worktree, name) || matches_dir_name(worktree, name)
            }
            RemoveMatchMode::Branch => matches_branch_name(worktree, name),
            RemoveMatchMode::Dir => matches_dir_name(worktree, name),
        })
        .collect();

    if matches.is_empty() {
        anyhow::bail!("worktree not found: {name}");
    }
    if matches.len() > 1 {
        let candidates = matches
            .iter()
            .map(|worktree| {
                let branch = worktree.branch.as_deref().unwrap_or("<none>");
                let dir = worktree_dir_name(worktree);
                format!(
                    "  - branch={branch} dir={dir} path={}",
                    worktree.path.display()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        anyhow::bail!(
            "ambiguous worktree name '{name}' matched multiple worktrees:\n{candidates}\nrerun with --match branch or --match dir"
        );
    }
    let wt = matches[0];

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

    println!(
        "{} Removed worktree at {}",
        style("✓").green().bold(),
        style(wt_path.display()).bold(),
    );

    if do_delete_branch && let Some(ref branch) = branch_name {
        delete_branch(&repo, branch)?;
        println!(
            "{} Deleted branch {}",
            style("✓").green().bold(),
            style(branch).cyan().bold(),
        );
    }

    Ok(())
}
