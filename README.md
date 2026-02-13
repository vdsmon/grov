# grov

An opinionated bare-repo-only Git worktree manager.

`grov` manages a predictable project layout:

- one bare repository at `repo.git`
- sibling working trees for branches
- optional branch-name prefixing

## Install

```sh
cargo install grov
```

Or download binaries from [GitHub Releases](https://github.com/vdsmon/grov/releases).

## Quick start

```sh
# Clone as bare and create the initial worktree
grov init --url https://github.com/user/project.git --prefix proj

# Enter the initial worktree
cd project/proj_main

# Create a worktree for a feature branch
grov add feature/login

# List worktrees
grov list

# Remove a worktree and delete its local branch
grov remove feature-login --delete-branch
```

## Command reference

### `grov init`

Clone repository as bare, write config, and create the initial worktree.

```sh
grov init --url https://github.com/user/repo.git --prefix rp
grov init --url https://github.com/user/repo.git --name myproject --prefix mp --branch develop
grov init
```

If flags are omitted, `init` prompts interactively for URL, name, prefix, and branch. The `--branch` flag overrides the auto-detected default branch.

### `grov add <branch>`

Create a new worktree, with branch resolution in this order:

1. existing local branch
2. existing remote branch (`origin/<branch>`) with tracking
3. new local branch — prompts for base branch (defaults to the current branch)

```sh
grov add feature/login
grov add hotfix --base release/1.0
grov add experimental --path /tmp/my-custom-worktree
```

Notes:

- `grov add` attempts `git fetch origin` first; fetch failures are warned and do not abort the command.
- When creating a new branch without `--base`, an interactive prompt asks for the base branch with the current branch as the default.
- In non-interactive contexts (scripts, CI), pass `--base` explicitly — stdin must be a terminal or the command exits with an error.

### `grov list` (alias: `grov ls`)

List non-bare worktrees and their state.

```sh
grov list
# ● main        ✓ clean    ↑2
# ○ feature-x   ✦ dirty    ↑1 ↓3
# ○ stale-wt    ! missing

# Compact output (branch names only)
grov list --compact
# main
# feature-x
```

Status tokens:

- `✓ clean`: no local changes
- `✦ dirty`: uncommitted changes
- `! missing`: worktree path no longer exists on disk
- `? unknown`: state could not be determined

`list` correctly marks the current worktree even when run from a nested subdirectory.

### `grov remove <name>` (alias: `grov rm`)

Remove a worktree by name.

```sh
grov remove feature-x
grov remove feature-x --force --delete-branch
grov remove feature-x --match branch
grov remove my-worktree-dir --match dir
```

Flags:

- `--match auto|branch|dir` (default: `auto`)
- `--force`
- `--delete-branch`

Ambiguity handling:

- `--match auto` matches by branch or directory name.
- if multiple candidates match, command exits with an ambiguity error and prints rerun guidance.

### `grov completions <shell>`

Generate shell completions.

```sh
grov completions bash >> ~/.bashrc
grov completions zsh >> ~/.zshrc
grov completions fish > ~/.config/fish/completions/grov.fish
```

## Project layout

grov enforces a sibling worktree layout:

```text
project/
├── repo.git/
│   ├── .grov.toml
│   ├── HEAD
│   ├── config
│   └── ...
├── proj_main/
├── proj_feature-login/
└── proj_dev/
```

If the prefix is blank, worktrees are named only by sanitized branch name.

## Development

Common local validation:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

For local dev setup with the in-development binary (`grovd`) in an isolated sandbox, see [docs/local-dev.md](docs/local-dev.md) (includes one-command zsh setup).

## CI and release automation

### CI (`.github/workflows/ci.yml`)

- triggers on push to `main` and pull requests
- runs `fmt`, `clippy`, and `test` as separate jobs
- uses Rust dependency caching and concurrency cancellation

### Release (`.github/workflows/release.yml`)

- triggers on tag push `v*`
- preflight gate runs fmt/clippy/test
- publishes to crates.io
- builds release binaries for Linux and macOS targets
- uploads release artifacts and `checksums.txt`
- publish job uses GitHub environment `release`

## Project governance

- branch protection on `main` requires checks: `fmt`, `clippy`, `test`
- conversation resolution is required before merge
- linear history enforced (no force-push/deletion)
- `CODEOWNERS`: `.github/CODEOWNERS`
- Dependabot weekly updates for Cargo and GitHub Actions: `.github/dependabot.yml`
- PR template: `.github/pull_request_template.md`
- issue templates: `.github/ISSUE_TEMPLATE/`

## Contributing and security

- Contribution guide: [CONTRIBUTING.md](CONTRIBUTING.md)
- Security policy: [SECURITY.md](SECURITY.md)

## License

MIT OR Apache-2.0
