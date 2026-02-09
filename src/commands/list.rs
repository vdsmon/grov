use console::style;

use crate::git::repo::find_bare_repo;
use crate::git::status;
use crate::git::worktree::list_worktrees;

enum WorktreeStatus {
    Clean,
    Dirty,
    Missing,
    Unknown,
}

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

    // Collect non-bare worktrees with their computed state
    let entries: Vec<_> = worktrees
        .iter()
        .filter(|wt| !wt.is_bare)
        .map(|wt| {
            let branch_name = wt
                .branch
                .clone()
                .unwrap_or_else(|| "(detached)".to_string());
            let wt_canonical = std::fs::canonicalize(&wt.path).ok();
            let is_current = cwd_canonical
                .as_ref()
                .zip(wt_canonical.as_ref())
                .map(|(cwd, root)| cwd == root || cwd.starts_with(root))
                .unwrap_or(false);
            let status = if !wt.path.exists() {
                WorktreeStatus::Missing
            } else {
                match status::is_dirty(&wt.path) {
                    Ok(true) => WorktreeStatus::Dirty,
                    Ok(false) => WorktreeStatus::Clean,
                    Err(_) => WorktreeStatus::Unknown,
                }
            };
            let ab = match status {
                WorktreeStatus::Clean | WorktreeStatus::Dirty => {
                    status::ahead_behind(&wt.path).unwrap_or(None)
                }
                WorktreeStatus::Missing | WorktreeStatus::Unknown => None,
            };
            let dir_name = wt
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            (branch_name, is_current, status, ab, dir_name)
        })
        .collect();

    if entries.is_empty() {
        println!("{}", style("No worktrees found.").dim());
        return Ok(());
    }

    // Find max branch name length for alignment
    let max_branch = entries.iter().map(|(b, ..)| b.len()).max().unwrap_or(0);

    for (branch_name, is_current, status, ab, dir_name) in &entries {
        // Marker + branch
        let (marker, branch_display) = if *is_current {
            (
                style("●").cyan().bold().to_string(),
                style(branch_name).cyan().bold().to_string(),
            )
        } else {
            (style("○").dim().to_string(), style(branch_name).to_string())
        };

        // Status indicator
        let status_str = match status {
            WorktreeStatus::Clean => style("✓ clean").green().to_string(),
            WorktreeStatus::Dirty => style("✦ dirty").yellow().to_string(),
            WorktreeStatus::Missing => style("! missing").red().to_string(),
            WorktreeStatus::Unknown => style("? unknown").yellow().to_string(),
        };

        // Ahead/behind
        let ab_str = format_ahead_behind(*ab);

        // Directory name in dim
        let path_str = style(format!("({dir_name})")).dim().to_string();

        let padded_branch = format!(
            "{branch_display}{}",
            " ".repeat(max_branch.saturating_sub(branch_name.len()))
        );

        println!("  {marker} {padded_branch}  {status_str}{ab_str}  {path_str}",);
    }

    Ok(())
}

fn format_ahead_behind(ab: Option<(u32, u32)>) -> String {
    match ab {
        Some((ahead, behind)) => {
            let mut parts = Vec::new();
            if ahead > 0 {
                parts.push(style(format!("↑{ahead}")).green().to_string());
            }
            if behind > 0 {
                parts.push(style(format!("↓{behind}")).red().to_string());
            }
            if parts.is_empty() {
                String::new()
            } else {
                format!("  {}", parts.join(" "))
            }
        }
        None => String::new(),
    }
}
