#![allow(deprecated)]

use std::process::Command;

use assert_cmd::Command as AssertCommand;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn init_clones_bare_repo() {
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
        .args(["init", source.to_str().unwrap()])
        .current_dir(&work_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized bare repo at"));

    // Verify bare repo was created
    let bare_path = work_dir.join("source.git");
    assert!(bare_path.exists());

    // Verify a worktree was created
    let trees_dir = bare_path.join("trees");
    assert!(trees_dir.exists());
}

#[test]
fn init_with_custom_name() {
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
        .args(["init", source.to_str().unwrap(), "--name", "myproject"])
        .current_dir(&work_dir)
        .assert()
        .success();

    assert!(work_dir.join("myproject.git").exists());
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
