#![allow(deprecated)]

mod common;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn add_new_branch() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    // Create initial worktree as sibling (test_main)
    let main_wt = project_dir.join("test_main");
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

    // Worktree should be at project_dir/test_test-branch
    assert!(project_dir.join("test_test-branch").exists());
}

#[test]
fn add_existing_local_branch() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
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

    assert!(project_dir.join("test_feature-x").exists());
}

#[test]
fn add_with_base_flag() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Create a base branch with a distinct commit
    std::fs::write(main_wt.join("base.txt"), "base content").unwrap();
    std::process::Command::new("git")
        .args(["-C", main_wt.to_str().unwrap(), "add", "."])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args([
            "-C",
            main_wt.to_str().unwrap(),
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            "base commit",
        ])
        .output()
        .unwrap();

    // Create a branch from the current state of main using --base
    Command::cargo_bin("grov")
        .unwrap()
        .args(["add", "from-base", "--base", "main"])
        .current_dir(&main_wt)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created worktree"));

    let wt_path = project_dir.join("test_from-base");
    assert!(wt_path.exists());
    // The file from the base commit should be present
    assert!(wt_path.join("base.txt").exists());
}

#[test]
fn add_duplicate_worktree_fails() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Create a worktree
    Command::cargo_bin("grov")
        .unwrap()
        .args(["add", "dupe-branch"])
        .current_dir(&main_wt)
        .assert()
        .success();

    // Try to add same branch again — should fail because dir exists
    Command::cargo_bin("grov")
        .unwrap()
        .args(["add", "dupe-branch"])
        .current_dir(&main_wt)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn add_with_custom_path() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    let custom_path = project_dir.join("my-custom-location");
    Command::cargo_bin("grov")
        .unwrap()
        .args([
            "add",
            "custom-branch",
            "--path",
            custom_path.to_str().unwrap(),
        ])
        .current_dir(&main_wt)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created worktree"));

    assert!(custom_path.exists());
}
