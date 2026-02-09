#![allow(deprecated)]

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn remove_worktree() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    // Create main worktree as sibling
    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Create a second worktree to remove
    let to_remove = project_dir.join("test_to-remove");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args([
            "worktree",
            "add",
            "-b",
            "to-remove",
            to_remove.to_str().unwrap(),
            "main",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());

    Command::cargo_bin("grov")
        .unwrap()
        .args(["remove", "to-remove"])
        .current_dir(&main_wt)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed worktree"));

    assert!(!to_remove.exists());
}

#[test]
fn remove_dirty_worktree_fails() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Create worktree and make it dirty
    let dirty_wt = project_dir.join("test_dirty-branch");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args([
            "worktree",
            "add",
            "-b",
            "dirty-branch",
            dirty_wt.to_str().unwrap(),
            "main",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Make it dirty
    std::fs::write(dirty_wt.join("dirty.txt"), "dirty").unwrap();

    Command::cargo_bin("grov")
        .unwrap()
        .args(["remove", "dirty-branch"])
        .current_dir(&main_wt)
        .assert()
        .failure()
        .stderr(predicate::str::contains("uncommitted changes"));
}

#[test]
fn remove_dirty_worktree_with_force() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    let dirty_wt = project_dir.join("test_dirty-branch");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args([
            "worktree",
            "add",
            "-b",
            "dirty-branch",
            dirty_wt.to_str().unwrap(),
            "main",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());

    std::fs::write(dirty_wt.join("dirty.txt"), "dirty").unwrap();

    Command::cargo_bin("grov")
        .unwrap()
        .args(["remove", "dirty-branch", "--force"])
        .current_dir(&main_wt)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed worktree"));
}

#[test]
fn remove_with_delete_branch() {
    let (_tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    let del_wt = project_dir.join("test_del-branch");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args([
            "worktree",
            "add",
            "-b",
            "del-branch",
            del_wt.to_str().unwrap(),
            "main",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());

    Command::cargo_bin("grov")
        .unwrap()
        .args(["remove", "del-branch", "--delete-branch"])
        .current_dir(&main_wt)
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted branch"));
}

#[test]
fn remove_ambiguous_auto_fails_with_candidates() {
    let (_tmp, main_wt, branch_wt, dir_wt) = setup_ambiguous_remove_case();

    Command::cargo_bin("grov")
        .unwrap()
        .args(["remove", "foo"])
        .current_dir(&main_wt)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("ambiguous worktree name 'foo'")
                .and(predicate::str::contains("branch=foo"))
                .and(predicate::str::contains("dir=foo"))
                .and(predicate::str::contains(
                    "rerun with --match branch or --match dir",
                )),
        );

    assert!(branch_wt.exists());
    assert!(dir_wt.exists());
}

#[test]
fn remove_ambiguous_match_branch_succeeds() {
    let (_tmp, main_wt, branch_wt, dir_wt) = setup_ambiguous_remove_case();

    Command::cargo_bin("grov")
        .unwrap()
        .args(["remove", "foo", "--match", "branch"])
        .current_dir(&main_wt)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed worktree"));

    assert!(!branch_wt.exists());
    assert!(dir_wt.exists());
}

#[test]
fn remove_ambiguous_match_dir_succeeds() {
    let (_tmp, main_wt, branch_wt, dir_wt) = setup_ambiguous_remove_case();

    Command::cargo_bin("grov")
        .unwrap()
        .args(["remove", "foo", "--match", "dir"])
        .current_dir(&main_wt)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed worktree"));

    assert!(branch_wt.exists());
    assert!(!dir_wt.exists());
}

fn setup_ambiguous_remove_case() -> (
    TempDir,
    std::path::PathBuf,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    let (tmp, bare, project_dir) = common::create_bare_repo();

    let main_wt = project_dir.join("test_main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    let branch_wt = project_dir.join("test_foo");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args([
            "worktree",
            "add",
            "-b",
            "foo",
            branch_wt.to_str().unwrap(),
            "main",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());

    let custom_root = tmp.path().join("custom");
    std::fs::create_dir_all(&custom_root).unwrap();
    let dir_wt = custom_root.join("foo");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args([
            "worktree",
            "add",
            "-b",
            "bar",
            dir_wt.to_str().unwrap(),
            "main",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());

    (tmp, main_wt, branch_wt, dir_wt)
}
