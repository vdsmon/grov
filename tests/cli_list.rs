#![allow(deprecated)]

mod common;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn list_in_bare_repo() {
    let (_tmp, bare) = common::create_bare_repo();

    // Create a worktree first
    let trees_dir = bare.join("trees");
    std::fs::create_dir_all(&trees_dir).unwrap();

    let wt_path = trees_dir.join("main");
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
    let (_tmp, bare) = common::create_bare_repo();

    let trees_dir = bare.join("trees");
    std::fs::create_dir_all(&trees_dir).unwrap();

    let wt_path = trees_dir.join("main");
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
