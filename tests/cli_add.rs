#![allow(deprecated)]

mod common;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn add_new_branch() {
    let (_tmp, bare) = common::create_bare_repo();

    // Create initial worktree so we have somewhere to run from
    let trees_dir = bare.join("trees");
    std::fs::create_dir_all(&trees_dir).unwrap();

    let main_wt = trees_dir.join("main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    Command::cargo_bin("grov")
        .unwrap()
        .args(["add", "test-branch"])
        .current_dir(&main_wt)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created worktree"));

    assert!(trees_dir.join("test-branch").exists());
}

#[test]
fn add_existing_local_branch() {
    let (_tmp, bare) = common::create_bare_repo();

    let trees_dir = bare.join("trees");
    std::fs::create_dir_all(&trees_dir).unwrap();

    let main_wt = trees_dir.join("main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Create a branch first
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["branch", "feature-x", "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    Command::cargo_bin("grov")
        .unwrap()
        .args(["add", "feature-x"])
        .current_dir(&main_wt)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created worktree"));

    assert!(trees_dir.join("feature-x").exists());
}
