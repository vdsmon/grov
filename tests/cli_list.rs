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

#[test]
fn list_marks_current_from_nested_subdir() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let wt_path = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", wt_path.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    let nested = wt_path.join("nested/deep");
    std::fs::create_dir_all(&nested).unwrap();

    Command::cargo_bin("grov")
        .unwrap()
        .args(["list"])
        .current_dir(&nested)
        .assert()
        .success()
        .stdout(predicate::str::contains("● main"));
}

#[test]
fn list_marks_missing_worktree() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    let missing_wt = project_dir.join("test_missing-branch");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args([
            "worktree",
            "add",
            "-b",
            "missing-branch",
            missing_wt.to_str().unwrap(),
            "main",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());

    std::fs::remove_dir_all(&missing_wt).unwrap();

    Command::cargo_bin("grov")
        .unwrap()
        .args(["list"])
        .current_dir(&main_wt)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("missing-branch").and(predicate::str::contains("! missing")),
        );
}
