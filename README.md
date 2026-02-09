# grov

An opinionated bare-repo-only git worktree manager.

## Install

```sh
cargo install grov
```

Or download a binary from [Releases](https://github.com/victordsm/grov/releases).

## Quick start

```sh
# Clone a repo as bare and create an initial worktree
grov init --url https://github.com/user/project.git --prefix proj

# Navigate into the worktree
cd project/proj_main

# Create a new worktree for a feature branch
grov add feature/login

# List all worktrees
grov list

# Remove a worktree (and its branch)
grov remove feature-login --delete-branch
```

## Commands

### `grov init`

Clones a repository as a bare repo and creates an initial worktree for the default branch. When flags are omitted, prompts interactively.

```sh
grov init --url https://github.com/user/repo.git --prefix rp
grov init --url https://github.com/user/repo.git --name myproject --prefix mp
grov init   # interactive: prompts for URL, name, and prefix
```

### `grov add <branch>`

Creates a new worktree. Intelligently resolves the branch:

- **Local branch exists** — checks it out in a new worktree
- **Remote branch exists** — creates a tracking local branch
- **Neither** — creates a new branch from `--base` (or the default branch)

```sh
grov add feature/login
grov add hotfix --base release/1.0
```

### `grov list` (alias: `ls`)

Lists all worktrees with status information.

```sh
grov list
# ● main        ✓ clean    ↑2
# ○ feature-x   ✦ dirty    ↑1 ↓3
# ○ stale-wt    ! missing

grov list --compact
# main
# feature-x
```

Status tokens:
- `✓ clean` — no local changes
- `✦ dirty` — uncommitted changes present
- `! missing` — worktree path no longer exists on disk
- `? unknown` — worktree status could not be determined

### `grov remove <name>` (alias: `rm`)

Removes a worktree. Refuses to remove dirty worktrees unless `--force` is used.

```sh
grov remove feature-x
grov remove feature-x --force --delete-branch
grov remove feature-x --match branch
```

When `<name>` could refer to multiple worktrees (for example branch match and directory-name match),
`grov remove` exits with an ambiguity error and asks you to rerun with one of:
- `--match branch`
- `--match dir`

### `grov completions <shell>`

Generates shell completions.

```sh
grov completions bash >> ~/.bashrc
grov completions zsh >> ~/.zshrc
grov completions fish > ~/.config/fish/completions/grov.fish
```

## How it works

grov enforces a sibling worktree layout with a project alias prefix:

```
project/                    # project directory (created by grov init)
├── repo.git/               # bare repository
│   ├── .grov.toml          # config (worktree prefix, etc.)
│   ├── HEAD
│   ├── config
│   └── ...
├── proj_main/              # <prefix>_<sanitized-branch>
├── proj_feature-login/     # <prefix>_<sanitized-branch>
└── proj_dev/
```

If the prefix is blank, worktrees are named by branch only (no prefix or underscore).

## License

MIT OR Apache-2.0
