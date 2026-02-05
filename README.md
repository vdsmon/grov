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
grov init https://github.com/user/project.git

# Navigate into the worktree
cd project.git/trees/main

# Create a new worktree for a feature branch
grov add feature/login

# List all worktrees
grov list

# Remove a worktree (and its branch)
grov remove feature-login --delete-branch
```

## Commands

### `grov init <url>`

Clones a repository as a bare repo and creates an initial worktree for the default branch.

```sh
grov init https://github.com/user/repo.git
grov init https://github.com/user/repo.git --name myproject
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
# * main        [clean] [↑2]
#   feature-x   [dirty] [↑1 ↓3]

grov list --compact
# main
# feature-x
```

### `grov remove <name>` (alias: `rm`)

Removes a worktree. Refuses to remove dirty worktrees unless `--force` is used.

```sh
grov remove feature-x
grov remove feature-x --force --delete-branch
```

### `grov completions <shell>`

Generates shell completions.

```sh
grov completions bash >> ~/.bashrc
grov completions zsh >> ~/.zshrc
grov completions fish > ~/.config/fish/completions/grov.fish
```

## How it works

grov enforces a bare-repo layout where worktrees live under `trees/`:

```
project.git/           # bare repository
├── trees/
│   ├── main/          # worktree for main branch
│   └── feature-login/ # worktree for feature/login branch
├── HEAD
├── config
└── ...
```

## License

MIT OR Apache-2.0
