use std::path::Path;

use console::style;

use crate::config::read_config;
use crate::git::executor::run_git_ok;
use crate::git::repo::{current_branch, default_branch, find_bare_repo};
use crate::git::worktree::{add_worktree, branch_exists_local, branch_exists_remote};
use crate::paths::worktree_dir;
use crate::ui::prompt;

#[derive(Debug, PartialEq)]
enum BaseBranchAction {
    UseBase(String),
    Prompt { default: Option<String> },
    ErrorNotTty,
}

fn resolve_base_branch(
    base_flag: Option<&str>,
    current_branch: Option<&str>,
    is_tty: bool,
) -> BaseBranchAction {
    if let Some(b) = base_flag {
        return BaseBranchAction::UseBase(b.to_string());
    }
    if !is_tty {
        return BaseBranchAction::ErrorNotTty;
    }
    BaseBranchAction::Prompt {
        default: current_branch.map(|s| s.to_string()),
    }
}

pub fn execute(branch: &str, base: Option<&str>, custom_path: Option<&Path>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let repo = find_bare_repo(&cwd)?;
    let config = read_config(&repo);

    // Fetch latest
    if let Err(err) = run_git_ok(Some(&repo), &["fetch", "origin"]) {
        eprintln!(
            "{} could not fetch from origin: {err:#}; continuing with local refs",
            style("warning:").yellow().bold()
        );
    }

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
        // New branch — resolve base via flag, prompt, or non-TTY error
        use std::io::IsTerminal;
        let current = current_branch(&cwd).unwrap_or(None);
        let is_tty = std::io::stdin().is_terminal();
        let base_branch = match resolve_base_branch(base, current.as_deref(), is_tty) {
            BaseBranchAction::UseBase(b) => b,
            BaseBranchAction::Prompt {
                default: prompt_default,
            } => {
                let fallback;
                let effective_default = match &prompt_default {
                    Some(b) => b.as_str(),
                    None => {
                        fallback = default_branch(&repo)?;
                        fallback.as_str()
                    }
                };
                let label = format!("Base branch for new branch '{branch}'");
                let input = prompt(
                    &label,
                    Some(effective_default),
                    &mut std::io::stdin().lock(),
                )?;
                if input.is_empty() {
                    effective_default.to_string()
                } else {
                    input
                }
            }
            BaseBranchAction::ErrorNotTty => {
                anyhow::bail!("--base is required when stdin is not a terminal");
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_base_provided() {
        assert_eq!(
            resolve_base_branch(Some("develop"), Some("feat"), true),
            BaseBranchAction::UseBase("develop".to_string())
        );
    }

    #[test]
    fn resolve_base_provided_overrides_non_tty() {
        assert_eq!(
            resolve_base_branch(Some("develop"), None, false),
            BaseBranchAction::UseBase("develop".to_string())
        );
    }

    #[test]
    fn resolve_tty_with_current_branch() {
        assert_eq!(
            resolve_base_branch(None, Some("feat"), true),
            BaseBranchAction::Prompt {
                default: Some("feat".to_string())
            }
        );
    }

    #[test]
    fn resolve_tty_without_current_branch() {
        assert_eq!(
            resolve_base_branch(None, None, true),
            BaseBranchAction::Prompt { default: None }
        );
    }

    #[test]
    fn resolve_non_tty_without_base() {
        assert_eq!(
            resolve_base_branch(None, Some("feat"), false),
            BaseBranchAction::ErrorNotTty
        );
    }

    #[test]
    fn resolve_non_tty_no_base_no_branch() {
        assert_eq!(
            resolve_base_branch(None, None, false),
            BaseBranchAction::ErrorNotTty
        );
    }
}
