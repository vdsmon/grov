#![allow(deprecated)]

mod common;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn remove_worktree() {
    let (_tmp, bare) = common::create_bare_repo();

    let trees_dir = bare.join("trees");
    std::fs::create_dir_all(&trees_dir).unwrap();

    // Create main worktree
    let main_wt = trees_dir.join("main");
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args(["worktree", "add", main_wt.to_str().unwrap(), "main"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Create a second worktree to remove
    let output = std::process::Command::new("git")
        .env("GIT_DIR", &bare)
        .args([
            "worktree",
            "add",
            "-b",
            "to-remove",
            trees_dir.join("to-remove").to_str().unwrap(),
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

    assert!(!trees_dir.join("to-remove").exists());
}

#[test]
fn remove_dirty_worktree_fails() {
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

    // Create worktree and make it dirty
    let dirty_wt = trees_dir.join("dirty-branch");
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

    let dirty_wt = trees_dir.join("dirty-branch");
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

    let del_wt = trees_dir.join("del-branch");
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
