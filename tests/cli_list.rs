#![allow(deprecated)]

mod common;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn list_in_worktree() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    // Create a worktree as sibling
    let wt_path = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", wt_path.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success(), "failed to add worktree");

    Command::cargo_bin("grov")
        .unwrap()
        .args(["list"])
        .current_dir(&wt_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));
}

#[test]
fn list_compact() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let wt_path = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", wt_path.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    Command::cargo_bin("grov")
        .unwrap()
        .args(["list", "--compact"])
        .current_dir(&wt_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));
}
