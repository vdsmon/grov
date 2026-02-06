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
fn list_shows_clean_for_unmodified_worktree() {
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
        .args(["list"])
        .current_dir(&wt_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("clean"));
}

#[test]
fn list_shows_dirty_for_modified_worktree() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let wt_path = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", wt_path.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Make the worktree dirty
    std::fs::write(wt_path.join("dirty.txt"), "dirty").unwrap();

    Command::cargo_bin("grov")
        .unwrap()
        .args(["list"])
        .current_dir(&wt_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("dirty"));
}

#[test]
fn list_shows_multiple_worktrees() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Create a second worktree
    let feature_wt = project_dir.join("test_feature");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args([
            "worktree",
            "add",
            "-b",
            "feature",
            feature_wt.to_str().unwrap(),
            "main",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());

    Command::cargo_bin("grov")
        .unwrap()
        .args(["list"])
        .current_dir(&main_wt)
        .assert()
        .success()
        .stdout(predicate::str::contains("main").and(predicate::str::contains("feature")));
}

#[test]
fn list_compact_omits_status() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let wt_path = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", wt_path.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Compact mode should not show [clean]/[dirty] markers
    let assert = Command::cargo_bin("grov")
        .unwrap()
        .args(["list", "--compact"])
        .current_dir(&wt_path)
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(!stdout.contains("clean"));
    assert!(!stdout.contains("dirty"));
    // Should just be the branch name
    assert_eq!(stdout.trim(), "main");
}
