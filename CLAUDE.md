<claude-mem-context>

</claude-mem-context>

# grov

Opinionated bare-repo-only git worktree manager (Rust CLI).

## Build & Test

- `cargo build` - build debug binary
- `cargo test` - run all tests (unit + integration)
- `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` - full CI check

## Architecture

- `src/main.rs` → error formatting with red `error:` prefix, calls `grov::run()`
- `src/lib.rs` → CLI parsing + command dispatch
- `src/cli.rs` → clap derive structs (Cli, Commands enum)
- `src/config.rs` → `GrovConfig` read/write for `.grov.toml` inside bare repo
- `src/commands/{init,add,list,remove,completions}.rs` → command implementations
- `src/git/executor.rs` → `run_git()` / `run_git_ok()` — single point for all git calls
- `src/git/repo.rs` → bare repo discovery (`find_bare_repo`), default branch detection
- `src/git/worktree.rs` → worktree CRUD, branch checks, porcelain parsing
- `src/git/status.rs` → dirty check, ahead/behind counts
- `src/paths.rs` → branch name sanitization, worktree dir paths, URL parsing
- `src/errors.rs` → `GrovError` thiserror enum

## Conventions

- Sibling worktree layout: `<project>/repo.git/` + `<project>/<prefix>_<branch>/`
- Config in `repo.git/.grov.toml` stores worktree prefix
- `worktree_dir(bare_repo, branch, prefix)` builds sibling path
- `find_bare_repo()` checks for `repo.git` child in current/parent dirs
- Edition 2021, MSRV 1.85, rustfmt edition 2024
- `run_git()` returns raw GitOutput; `run_git_ok()` errors on non-zero exit
- `add_worktree()` signature: `(repo, path, commit_ish: Option<&str>, extra_args: &[&str])`
- Use `GrovError` for domain errors (bad repo, missing branch); `anyhow` for command-level orchestration
- New commands: add file in `src/commands/`, variant in `Commands` enum in `cli.rs`, dispatch arm in `lib.rs`

## Gotchas

- rustfmt edition 2024 reformats aggressively — always run `cargo fmt` before `cargo clippy`
- `assert_cmd` 2.x deprecated `cargo_bin`; test crate roots need `#![allow(deprecated)]`

## Testing

- Unit tests in `src/paths.rs` and `src/git/worktree.rs`
- Integration tests in `tests/cli_{init,add,list,remove}.rs` using assert_cmd + tempfile
- `tests/common/mod.rs` has `create_bare_repo()` helper — returns `(TempDir, bare_path, project_dir)`, uses `#![allow(dead_code)]`
