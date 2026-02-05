#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

/// Create a temporary project with a bare repo at `tmp/project/repo.git`,
/// a `.grov.toml` with prefix "test", and the source repo for cloning.
///
/// Returns (TempDir, bare_repo_path, project_dir).
pub fn create_bare_repo() -> (TempDir, PathBuf, PathBuf) {
    create_bare_repo_with_prefix("test")
}

/// Create a temporary project with a configurable prefix.
pub fn create_bare_repo_with_prefix(prefix: &str) -> (TempDir, PathBuf, PathBuf) {
    let tmp = TempDir::new().expect("failed to create temp dir");

    // Create a normal repo first, make a commit, then clone it bare
    let source = tmp.path().join("source");
    std::fs::create_dir_all(&source).unwrap();

    run(&source, &["git", "init", "-b", "main"]);
    run(&source, &["git", "config", "user.email", "test@test.com"]);
    run(&source, &["git", "config", "user.name", "Test"]);

    // Create initial commit
    let file = source.join("README.md");
    std::fs::write(&file, "# test\n").unwrap();
    run(&source, &["git", "add", "."]);
    run(&source, &["git", "commit", "-m", "initial"]);

    // Create project directory
    let project_dir = tmp.path().join("project");
    std::fs::create_dir_all(&project_dir).unwrap();

    // Clone as bare into project/repo.git
    let bare = project_dir.join("repo.git");
    run(
        tmp.path(),
        &[
            "git",
            "clone",
            "--bare",
            source.to_str().unwrap(),
            bare.to_str().unwrap(),
        ],
    );

    // Fix fetch refspec
    run_with_env(
        &bare,
        &[
            "git",
            "config",
            "remote.origin.fetch",
            "+refs/heads/*:refs/remotes/origin/*",
        ],
    );

    // Set up symbolic ref for default branch detection
    run_with_env(
        &bare,
        &[
            "git",
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );

    // Write .grov.toml
    let config_content = format!("[worktree]\nprefix = \"{prefix}\"\n");
    std::fs::write(bare.join(".grov.toml"), config_content).unwrap();

    (tmp, bare, project_dir)
}

fn run(dir: &Path, args: &[&str]) {
    let status = Command::new(args[0])
        .args(&args[1..])
        .current_dir(dir)
        .output()
        .expect("failed to run command");
    assert!(
        status.status.success(),
        "command failed: {:?}\nstderr: {}",
        args,
        String::from_utf8_lossy(&status.stderr)
    );
}

fn run_with_env(bare_path: &Path, args: &[&str]) {
    let status = Command::new(args[0])
        .args(&args[1..])
        .env("GIT_DIR", bare_path)
        .output()
        .expect("failed to run command");
    assert!(
        status.status.success(),
        "command failed: {:?}\nstderr: {}",
        args,
        String::from_utf8_lossy(&status.stderr)
    );
}
