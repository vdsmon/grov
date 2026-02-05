use console::style;

use crate::git::repo::find_bare_repo;
use crate::git::status;
use crate::git::worktree::list_worktrees;

pub fn execute(compact: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let repo = find_bare_repo(&cwd)?;
    let worktrees = list_worktrees(&repo)?;

    // Determine current worktree
    let cwd_canonical = std::fs::canonicalize(&cwd).ok();

    if compact {
        for wt in &worktrees {
            if wt.is_bare {
                continue;
            }
            if let Some(ref branch) = wt.branch {
                println!("{branch}");
            }
        }
        return Ok(());
    }

    // Find max branch name length for alignment
    let max_len = worktrees
        .iter()
        .filter(|wt| !wt.is_bare)
        .filter_map(|wt| wt.branch.as_ref())
        .map(|b| b.len())
        .max()
        .unwrap_or(0);

    for wt in &worktrees {
        if wt.is_bare {
            continue;
        }

        let branch_name = wt.branch.as_deref().unwrap_or("(detached)");
        let is_current = cwd_canonical
            .as_ref()
            .and_then(|cwd| std::fs::canonicalize(&wt.path).ok().map(|p| p == *cwd))
            .unwrap_or(false);

        let marker = if is_current { "*" } else { " " };

        // Status info
        let dirty = status::is_dirty(&wt.path).unwrap_or(false);
        let ab = status::ahead_behind(&wt.path).unwrap_or(None);

        let status_str = if dirty {
            style("[dirty]").yellow().to_string()
        } else {
            style("[clean]").green().to_string()
        };

        let ab_str = format_ahead_behind(ab);

        println!("{marker} {branch_name:<max_len$}  {status_str}{ab_str}",);
    }

    Ok(())
}

fn format_ahead_behind(ab: Option<(u32, u32)>) -> String {
    match ab {
        Some((ahead, behind)) => {
            let mut parts = Vec::new();
            if ahead > 0 {
                parts.push(format!("↑{ahead}"));
            }
            if behind > 0 {
                parts.push(format!("↓{behind}"));
            }
            if parts.is_empty() {
                String::new()
            } else {
                format!(" {}", style(format!("[{}]", parts.join(" "))).dim())
            }
        }
        None => String::new(),
    }
}
