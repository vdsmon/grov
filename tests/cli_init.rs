#![allow(deprecated)]

use std::process::Command;

use assert_cmd::Command as AssertCommand;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn init_creates_project_with_bare_repo_and_worktree() {
    let tmp = TempDir::new().unwrap();

    // Create a source repo to clone from
    let source = tmp.path().join("source");
    std::fs::create_dir_all(&source).unwrap();
    run(&source, &["git", "init", "-b", "main"]);
    run(&source, &["git", "config", "user.email", "test@test.com"]);
    run(&source, &["git", "config", "user.name", "Test"]);
    std::fs::write(source.join("README.md"), "# test\n").unwrap();
    run(&source, &["git", "add", "."]);
    run(&source, &["git", "commit", "-m", "initial"]);

    let work_dir = tmp.path().join("work");
    std::fs::create_dir_all(&work_dir).unwrap();

    AssertCommand::cargo_bin("grov")
        .unwrap()
        .args([
            "init",
            "--url",
            source.to_str().unwrap(),
            "--name",
            "myproject",
            "--prefix",
            "mp",
            "--branch",
            "main",
        ])
        .current_dir(&work_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized myproject/"));

    // Verify project dir created
    let project_dir = work_dir.join("myproject");
    assert!(project_dir.exists());

    // Verify bare repo inside project dir
    let bare_path = project_dir.join("repo.git");
    assert!(bare_path.exists());

    // Verify .grov.toml written
    assert!(bare_path.join(".grov.toml").exists());

    // Verify worktree created as sibling with prefix
    let wt_path = project_dir.join("mp_main");
    assert!(wt_path.exists());
}

#[test]
fn init_with_empty_prefix() {
    let tmp = TempDir::new().unwrap();

    let source = tmp.path().join("source");
    std::fs::create_dir_all(&source).unwrap();
    run(&source, &["git", "init", "-b", "main"]);
    run(&source, &["git", "config", "user.email", "test@test.com"]);
    run(&source, &["git", "config", "user.name", "Test"]);
    std::fs::write(source.join("README.md"), "# test\n").unwrap();
    run(&source, &["git", "add", "."]);
    run(&source, &["git", "commit", "-m", "initial"]);

    let work_dir = tmp.path().join("work");
    std::fs::create_dir_all(&work_dir).unwrap();

    AssertCommand::cargo_bin("grov")
        .unwrap()
        .args([
            "init",
            "--url",
            source.to_str().unwrap(),
            "--name",
            "myproject",
            "--prefix",
            "",
            "--branch",
            "main",
        ])
        .current_dir(&work_dir)
        .assert()
        .success();

    // Worktree should be just the branch name (no prefix)
    let project_dir = work_dir.join("myproject");
    assert!(project_dir.join("main").exists());
}

#[test]
fn init_with_custom_branch() {
    let tmp = TempDir::new().unwrap();

    let source = tmp.path().join("source");
    std::fs::create_dir_all(&source).unwrap();
    run(&source, &["git", "init", "-b", "main"]);
    run(&source, &["git", "config", "user.email", "test@test.com"]);
    run(&source, &["git", "config", "user.name", "Test"]);
    std::fs::write(source.join("README.md"), "# test\n").unwrap();
    run(&source, &["git", "add", "."]);
    run(&source, &["git", "commit", "-m", "initial"]);
    // Create a develop branch
    run(&source, &["git", "checkout", "-b", "develop"]);
    run(&source, &["git", "checkout", "main"]);

    let work_dir = tmp.path().join("work");
    std::fs::create_dir_all(&work_dir).unwrap();

    AssertCommand::cargo_bin("grov")
        .unwrap()
        .args([
            "init",
            "--url",
            source.to_str().unwrap(),
            "--name",
            "myproject",
            "--prefix",
            "mp",
            "--branch",
            "develop",
        ])
        .current_dir(&work_dir)
        .assert()
        .success();

    // Verify worktree created for develop branch
    let project_dir = work_dir.join("myproject");
    assert!(project_dir.join("mp_develop").exists());
}

#[test]
fn init_output_contains_both_paths() {
    let tmp = TempDir::new().unwrap();

    let source = tmp.path().join("source");
    std::fs::create_dir_all(&source).unwrap();
    run(&source, &["git", "init", "-b", "main"]);
    run(&source, &["git", "config", "user.email", "test@test.com"]);
    run(&source, &["git", "config", "user.name", "Test"]);
    std::fs::write(source.join("README.md"), "# test\n").unwrap();
    run(&source, &["git", "add", "."]);
    run(&source, &["git", "commit", "-m", "initial"]);

    let work_dir = tmp.path().join("work");
    std::fs::create_dir_all(&work_dir).unwrap();

    AssertCommand::cargo_bin("grov")
        .unwrap()
        .args([
            "init",
            "--url",
            source.to_str().unwrap(),
            "--name",
            "myproject",
            "--prefix",
            "mp",
            "--branch",
            "main",
        ])
        .current_dir(&work_dir)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("myproject/repo.git")
                .and(predicate::str::contains("myproject/mp_main")),
        );
}

#[test]
fn init_prints_cd_hints() {
    let tmp = TempDir::new().unwrap();

    let source = tmp.path().join("source");
    std::fs::create_dir_all(&source).unwrap();
    run(&source, &["git", "init", "-b", "main"]);
    run(&source, &["git", "config", "user.email", "test@test.com"]);
    run(&source, &["git", "config", "user.name", "Test"]);
    std::fs::write(source.join("README.md"), "# test\n").unwrap();
    run(&source, &["git", "add", "."]);
    run(&source, &["git", "commit", "-m", "initial"]);

    let work_dir = tmp.path().join("work");
    std::fs::create_dir_all(&work_dir).unwrap();

    AssertCommand::cargo_bin("grov")
        .unwrap()
        .args([
            "init",
            "--url",
            source.to_str().unwrap(),
            "--name",
            "myproject",
            "--prefix",
            "mp",
            "--branch",
            "main",
        ])
        .current_dir(&work_dir)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("To enter the project:")
                .and(predicate::str::contains("cd myproject"))
                .and(predicate::str::contains("To start working:"))
                .and(predicate::str::contains("cd myproject/mp_main")),
        );
}

fn run(dir: &std::path::Path, args: &[&str]) {
    let output = Command::new(args[0])
        .args(&args[1..])
        .current_dir(dir)
        .output()
        .expect("failed to run command");
    assert!(
        output.status.success(),
        "command failed: {:?}\nstderr: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}
