use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::config::{GrovConfig, WorktreeConfig, write_config};
use crate::git::executor::run_git_ok;
use crate::git::repo::default_branch;
use crate::git::worktree::add_worktree;
use crate::paths::{repo_name_from_url, worktree_dir};

pub fn execute(
    url: Option<&str>,
    name: Option<&str>,
    prefix: Option<&str>,
    path: Option<&Path>,
) -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    // 1. URL — use flag or prompt
    let url = match url {
        Some(u) => u.to_string(),
        None => {
            eprint!("Repository URL: ");
            io::stderr().flush()?;
            let mut line = String::new();
            reader.read_line(&mut line)?;
            let line = line.trim().to_string();
            if line.is_empty() {
                anyhow::bail!("URL is required");
            }
            line
        }
    };

    // 2. Project name — derive from URL, allow override
    let derived_name = repo_name_from_url(&url);
    let project_name = match name {
        Some(n) => n.to_string(),
        None => {
            eprint!("Project name [{}]: ", derived_name);
            io::stderr().flush()?;
            let mut line = String::new();
            reader.read_line(&mut line)?;
            let line = line.trim().to_string();
            if line.is_empty() { derived_name } else { line }
        }
    };

    // 3. Prefix — use flag or prompt
    let prefix = match prefix {
        Some(p) => p.to_string(),
        None => {
            eprint!("Worktree prefix (e.g. short alias, blank for none) []: ");
            io::stderr().flush()?;
            let mut line = String::new();
            reader.read_line(&mut line)?;
            line.trim().to_string()
        }
    };

    let parent = match path {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir()?,
    };

    // Create project directory (like git clone creates a directory)
    let project_dir = parent.join(&project_name);
    if project_dir.exists() {
        anyhow::bail!("directory already exists: {}", project_dir.display());
    }
    std::fs::create_dir_all(&project_dir)?;

    // Clone bare into <project>/repo.git
    let bare_path = project_dir.join("repo.git");
    let bare_str = bare_path.to_string_lossy().to_string();
    run_git_ok(None, &["clone", "--bare", &url, &bare_str])?;

    // Write .grov.toml
    let config = GrovConfig {
        worktree: WorktreeConfig {
            prefix: prefix.clone(),
        },
    };
    write_config(&bare_path, &config)?;

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

    // Create initial worktree as sibling of repo.git
    let wt_path = worktree_dir(&bare_path, &branch, &prefix);
    add_worktree(&bare_path, &wt_path, Some(&branch), &[])?;

    let wt_dir_name = wt_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    println!("Initialized {project_name}/ with worktree {wt_dir_name}");
    Ok(())
}
