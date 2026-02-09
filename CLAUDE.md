<claude-mem-context>

</claude-mem-context>

# grov - Claude Project Notes

Rust CLI for managing Git worktrees around a bare repo layout.

## Project Snapshot

- Package: `grov` (`edition = "2024"`, `rust-version = "1.93"`)
- Primary use case: initialize and operate a sibling worktree layout rooted at `<project>/repo.git`
- License: MIT OR Apache-2.0

## Core CLI Behavior

- `grov init`
  - clones a bare repo into `<project>/repo.git`
  - writes `repo.git/.grov.toml`
  - creates initial worktree for default branch
  - supports interactive prompts if flags are omitted

- `grov add <branch>`
  - resolution order:
    1. existing local branch
    2. existing remote branch (`origin/<branch>`) with tracking
    3. new branch from `--base` or detected default branch
  - attempts `git fetch origin` first
  - fetch failures are warnings (non-fatal); command continues with local refs

- `grov list` (`grov ls`)
  - full view prints marker, branch, status, ahead/behind, and dir name
  - compact view prints branch names only
  - status tokens:
    - `✓ clean`
    - `✦ dirty`
    - `! missing`
    - `? unknown`
  - current marker detection works when run from nested subdirectories inside a worktree

- `grov remove <name>` (`grov rm`)
  - default matching mode `--match auto`
  - explicit modes:
    - `--match branch`
    - `--match dir`
  - `auto` fails on ambiguity and prints candidate list + rerun hint
  - dirty worktrees require `--force`
  - optional `--delete-branch`

- `grov completions <shell>`
  - generates shell completion scripts via clap

## Development Commands

Use these as the local pre-PR baseline:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Architecture Map

- `src/main.rs`: top-level error formatting and exit code
- `src/lib.rs`: CLI parse + dispatch
- `src/cli.rs`: clap commands/flags (`RemoveMatchMode` lives here)
- `src/config.rs`: `.grov.toml` read/write for bare repo config
- `src/commands/*.rs`: command handlers (`init`, `add`, `list`, `remove`, `completions`)
- `src/git/executor.rs`: shared git command wrapper (`run_git`, `run_git_ok`)
- `src/git/repo.rs`: bare repo discovery + default branch detection
- `src/git/worktree.rs`: porcelain parsing, worktree CRUD, branch/directory matching helpers
- `src/git/status.rs`: dirty and ahead/behind status
- `src/paths.rs`: branch sanitization + worktree naming
- `src/errors.rs`: domain error enum (`GrovError`)

## Conventions

- Sibling layout: `<project>/repo.git/` + `<project>/<prefix>_<branch>/`
- Config location: `repo.git/.grov.toml`
- Path helper: `worktree_dir(bare_repo, branch, prefix)`
- Bare repo discovery: `find_bare_repo()` looks in current context and parent layout
- Edition/MSRV: Rust 2024 + 1.93

## Tests

- Unit tests:
  - `src/paths.rs`
  - `src/git/worktree.rs`
- Integration tests:
  - `tests/cli_init.rs`
  - `tests/cli_add.rs`
  - `tests/cli_list.rs`
  - `tests/cli_remove.rs`
- Shared harness: `tests/common/mod.rs`

## Automation and Release

- CI workflow: `.github/workflows/ci.yml`
  - triggers on `push` to `main` and `pull_request`
  - required jobs: `fmt`, `clippy`, `test`
  - uses Rust cache and concurrency cancellation

- Release workflow: `.github/workflows/release.yml`
  - triggers on tag push `v*`
  - `preflight` gate (fmt/clippy/test)
  - `publish` job uses environment `release`
  - cross-platform builds and artifact packaging
  - attaches tarballs and `checksums.txt` to GitHub Release
  - requires `CARGO_REGISTRY_TOKEN` in repo secrets

## Repository Governance (Current)

- `main` branch protection requires:
  - status checks: `fmt`, `clippy`, `test`
  - conversation resolution
  - linear history
  - no force pushes or deletions
- CODEOWNERS: `.github/CODEOWNERS`
- Dependabot config: `.github/dependabot.yml` (weekly Cargo + GitHub Actions updates)
- PR template: `.github/pull_request_template.md`
- Issue templates: `.github/ISSUE_TEMPLATE/*`
- Security policy: `SECURITY.md`

## Gotchas

- `Cargo.lock` is intentionally gitignored, so avoid `--locked` in CI commands.
- Some automation checks (for example CodeRabbit) may appear on PRs but are not required merge gates.
- `assert_cmd` 2.x uses deprecated `cargo_bin`; tests currently allow deprecation where needed.
