# Spec: Print cd hint after worktree creation

> Status: done

## User Story

As a grov user, I want a copy-pasteable `cd` command printed after `grov init` and `grov add` so that I can jump into the new worktree without manually constructing the path.

## Context

Both `grov init` and `grov add` already print success output, but the user must manually figure out the path to `cd` into. Adding a dimmed `cd` hint makes the workflow seamless. The path should always be relative to the current working directory for brevity.

## Acceptance Criteria

- [x] `grov init` prints two dimmed cd hints after success: one for the project root, one for the worktree
- [x] `grov add` prints one dimmed cd hint after success pointing to the new worktree
- [x] All cd paths are relative to the user's current working directory
- [x] cd hints are only printed when the user's cwd differs from the target directory
- [x] cd hints use sentence-style labels (e.g., "To enter the project:", "To start working:")
- [x] cd hints are visually distinct from the main success output (dimmed styling)
- [x] Existing success output format is not changed

## Technical Design

### Affected Files

- `src/commands/init.rs` — add two cd hint lines after the existing `println!` block
- `src/commands/add.rs` — add one cd hint line after the existing `println!`
- `src/paths.rs` — add a helper function to compute a relative path from cwd to target

### Approach

**Relative path helper**: Add a `relative_from(target: &Path, base: &Path)` function to `src/paths.rs` that computes a relative path from `base` to `target`. Use `std::path::Path` methods — strip the common prefix and prepend `..` segments as needed. If both paths aren't absolute, canonicalize first. Fall back to the absolute path if relative computation fails.

**`grov init` changes** (src/commands/init.rs, after line 131):

After the existing success block, compute relative paths from `cwd` (which is `parent`, the directory where `grov init` was run) to:
1. `project_dir` — the project root
2. `wt_path` — the initial worktree

Print two dimmed lines with sentence labels:
```text
  To enter the project:  cd project/
  To start working:      cd project/prefix_main
```

Only print each line if cwd != target. In practice, cwd will always differ from both targets (since init creates a new directory), but the check keeps behavior correct.

**`grov add` changes** (src/commands/add.rs, after line 113):

Compute relative path from `cwd` to `wt_path`. Print one dimmed line:
```text
  To start working:  cd prefix_branch
```

Only print if cwd != wt_path.

**Styling**: Use `console::style(...).dim()` for the entire cd hint lines, consistent with existing use of the `console` crate.

### Edge Cases

- **cwd is already the target**: Skip the cd hint for that target (e.g., if somehow running `grov add` from the exact worktree path that would be created — unlikely but handled)
- **Path with spaces**: The cd hint should quote the path if it contains spaces (e.g., `cd "my project/prefix_main"`)
- **`grov init --path`**: When a custom path is provided, the relative path computation should still work relative to cwd, not the custom path
- **Symlinks/canonicalization**: Use `std::fs::canonicalize` for the cwd-vs-target comparison to avoid false negatives from symlinks, but use the non-canonicalized relative path for display (cleaner output)

## Tasks

- [x] **Task 1**: Add `relative_from(target, base)` helper to `src/paths.rs` with unit tests. The function takes two absolute paths and returns a relative path string from `base` to `target`. Include tests for: same directory (returns `.`), child directory, sibling directory, deeply nested paths, and fallback behavior.
- [x] **Task 2**: Add cd hint output to `grov add` in `src/commands/add.rs`. After the existing success `println!`, compute relative path from `cwd` to `wt_path`, and if cwd != wt_path, print a dimmed line: `To start working:  cd <relative_path>`. Quote the path if it contains spaces.
- [x] **Task 3**: Add cd hint output to `grov init` in `src/commands/init.rs`. After the existing success block, compute relative paths from `parent` (the original cwd) to both `project_dir` and `wt_path`. Print up to two dimmed lines with sentence labels. Only print each line if cwd != target.
- [x] **Task 4**: Add integration tests in `tests/cli_init.rs` and `tests/cli_add.rs` verifying the cd hint appears in stdout after successful worktree creation, uses relative paths, and uses the expected label text.

## Testing

- [x] Unit tests for `relative_from()` in `src/paths.rs` covering same dir, child, sibling, deep nesting, and paths with spaces
- [x] Integration test for `grov init` verifying both cd hint lines appear with correct relative paths and labels
- [x] Integration test for `grov add` verifying the cd hint line appears with correct relative path and label
- [ ] Manual verification that hints render as dimmed in a real terminal

## Out of Scope

- Shell-specific `cd` syntax (e.g., `pushd`) — always use plain `cd`
- Auto-changing directory (this is just a hint, not an action)
- Configurable hint format or ability to disable the hint
- Fish/zsh-specific path quoting — standard POSIX quoting is sufficient
