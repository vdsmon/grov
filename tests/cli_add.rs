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
        .args(["add", "test-branch", "--base", "main"])
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
fn add_warns_when_fetch_fails_but_continues() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["branch", "local-only", "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["remote", "set-url", "origin", "/definitely/does/not/exist"])
        .output()
        .unwrap();
    assert!(output.status.success());

    Command::cargo_bin("grov")
        .unwrap()
        .args(["add", "local-only"])
        .current_dir(&main_wt)
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "warning: could not fetch from origin",
        ));

    assert!(project_dir.join("test_local-only").exists());
}

#[test]
fn add_new_branch_non_tty_without_base_fails() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    Command::cargo_bin("grov")
        .unwrap()
        .args(["add", "new-branch"])
        .current_dir(&main_wt)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--base is required when stdin is not a terminal",
        ));
}

#[test]
fn add_new_branch_with_base_bypasses_prompt() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    Command::cargo_bin("grov")
        .unwrap()
        .args(["add", "new-branch", "--base", "main"])
        .current_dir(&main_wt)
        .assert()
        .success()
        .stderr(predicate::str::contains("Base branch").not());

    assert!(project_dir.join("test_new-branch").exists());
}

#[test]
fn add_existing_local_branch_no_prompt() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Create a local branch
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["branch", "local-feature", "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    Command::cargo_bin("grov")
        .unwrap()
        .args(["add", "local-feature"])
        .current_dir(&main_wt)
        .assert()
        .success()
        .stderr(predicate::str::contains("Base branch").not());

    assert!(project_dir.join("test_local-feature").exists());
}

#[test]
fn add_prints_cd_hint() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Create a local branch
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["branch", "hint-branch", "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    Command::cargo_bin("grov")
        .unwrap()
        .args(["add", "hint-branch"])
        .current_dir(&main_wt)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("To start working:")
                .and(predicate::str::contains("cd"))
                .and(predicate::str::contains("test_hint-branch")),
        );
}

#[test]
fn add_no_branch_non_tty_fails() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    Command::cargo_bin("grov")
        .unwrap()
        .args(["add"])
        .current_dir(&main_wt)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "branch argument is required when stdin is not a terminal",
        ));
}

#[test]
fn add_existing_remote_branch_no_prompt() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Create a branch in the source repo so it appears as a remote branch after fetch
    let source = _tmp.path().join("source");
    let output = std::process::Command::new("git")
        .current_dir(&source)
        .args(["branch", "remote-feature", "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    Command::cargo_bin("grov")
        .unwrap()
        .args(["add", "remote-feature"])
        .current_dir(&main_wt)
        .assert()
        .success()
        .stderr(predicate::str::contains("Base branch").not());

    assert!(project_dir.join("test_remote-feature").exists());
}
