use std::path::Path;
use std::process::{Command, ExitStatus};

use crate::errors::GrovError;

pub struct GitOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus,
}

/// Run a git command in the context of a repository.
/// Sets `GIT_DIR` to `repo_path` if provided.
pub fn run_git(repo_path: Option<&Path>, args: &[&str]) -> crate::errors::Result<GitOutput> {
    let mut cmd = Command::new("git");

    if let Some(path) = repo_path {
        cmd.env("GIT_DIR", path);
    }

    cmd.args(args);

    let output = cmd.output()?;

    Ok(GitOutput {
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        status: output.status,
    })
}

/// Run a git command and return stdout if successful, or error if non-zero exit.
pub fn run_git_ok(repo_path: Option<&Path>, args: &[&str]) -> crate::errors::Result<String> {
    let output = run_git(repo_path, args)?;

    if output.status.success() {
        Ok(output.stdout)
    } else {
        let msg = if output.stderr.is_empty() {
            format!("git {} exited with {}", args.join(" "), output.status)
        } else {
            output.stderr
        };
        Err(GrovError::GitCommandFailed(msg))
    }
}
