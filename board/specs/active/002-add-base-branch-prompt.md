# Spec: Add base branch prompt to `grov add`

> Status: draft

## User Story

As a developer, I want `grov add` to prompt me for the base branch when creating a new branch so that I can branch off the correct starting point instead of always getting the repo's default branch.

## Context

Currently, `grov add <branch>` silently uses the repo's default branch (e.g., `main`) as the base when creating a new branch that doesn't exist locally or remotely. The `--base` flag exists but is easy to forget. Since users typically want to branch off whatever they're currently working on, an interactive prompt with the current branch as the default is more ergonomic.

Source: `board/todo/add-base-branch-prompt.md` (consumed)

## Acceptance Criteria

- [ ] When creating a new branch (not existing locally or remotely) without `--base`, `grov add` prompts: `? Base branch [<default>]: `
- [ ] The prompt default is the current branch of the cwd's worktree (detected via `git rev-parse --abbrev-ref HEAD`)
- [ ] If current branch detection fails (e.g., running from `repo.git` or project root), the prompt default falls back to `default_branch()`
- [ ] If `--base` is provided, no prompt appears (existing behavior preserved)
- [ ] If stdin is not a TTY and `--base` is not provided, the command errors with a message telling the user to pass `--base`
- [ ] The prompt uses the same styling as `grov init` prompts (cyan `?`, bold label, dim default in brackets)
- [ ] Existing behavior for local and remote branch checkout is unchanged (no prompt in those paths)

## Technical Design

### Affected Files

- `src/commands/add.rs` — add interactive prompt logic for the new-branch path
- `src/commands/init.rs` — extract the `prompt()` helper so it can be shared
- `src/lib.rs` — no changes expected (dispatch already passes `base`)
- `src/cli.rs` — no changes (the `--base` flag stays as-is)

### Approach

1. **Extract the `prompt()` function** from `src/commands/init.rs` into a shared module (e.g., `src/ui.rs` or `src/prompt.rs`) so both `init` and `add` can use it.

2. **Add current-branch detection** — a new helper function (e.g., `current_branch(cwd)`) that runs `git rev-parse --abbrev-ref HEAD` from the given directory. Returns `Option<String>` — `None` on failure (detached HEAD, not in a worktree, etc.).

3. **Add TTY detection** — before prompting, check `std::io::stdin().is_terminal()` (available via `std::io::IsTerminal` in Rust 1.70+). If not a TTY and `--base` is absent, return an error.

4. **Update the new-branch path in `add::execute()`** — when neither local nor remote branch exists and `--base` is `None`:
   - Detect current branch from cwd
   - Fall back to `default_branch(&repo)?`
   - If TTY: prompt with detected default
   - If not TTY: error

### Edge Cases

- **Detached HEAD**: `git rev-parse --abbrev-ref HEAD` returns `HEAD` literally. Treat this as "no current branch" and fall back to `default_branch()`.
- **Running from `repo.git` directory**: No worktree context, so `rev-parse` will likely fail or return something unhelpful. Fall back to `default_branch()`.
- **Running from project root** (not inside a worktree): Same as above — fall back.
- **`default_branch()` also fails**: The existing error propagation handles this (returns `GrovError`).
- **User enters empty string at prompt**: Use the displayed default (same pattern as `grov init`).
- **User enters a non-existent base branch**: Let git handle the error naturally when `add_worktree()` is called.

## Tasks

<!-- Each task should be small enough for one agent session -->

- [ ] **Task 1**: Extract `prompt()` from `src/commands/init.rs` into `src/ui.rs`. Create `src/ui.rs` with the shared `prompt()` function. Update `init.rs` to import from `src/ui.rs`. Add `mod ui;` to `src/lib.rs`. Verify `cargo test` passes.
- [ ] **Task 2**: Add `current_branch()` helper. Create a `current_branch(cwd: &Path) -> Option<String>` function in `src/git/repo.rs` that runs `git rev-parse --abbrev-ref HEAD` from the given directory. Return `None` for failures or detached HEAD (`"HEAD"` literal).
- [ ] **Task 3**: Add interactive base-branch prompt to `grov add`. In `src/commands/add.rs`, when the branch is new and `--base` is `None`: detect current branch, fall back to `default_branch()`, check TTY, prompt or error. Use the shared `prompt()` from `src/ui.rs`.
- [ ] **Task 4**: Add integration tests. Test the non-TTY error case (no `--base`, piped stdin). Test the `--base` flag still works without prompting. Test that existing local/remote branch paths are unaffected.

## Testing

- [ ] Unit test for `current_branch()` — detached HEAD returns `None`
- [ ] Integration test: `grov add new-branch` with piped stdin (no TTY) and no `--base` should error
- [ ] Integration test: `grov add new-branch --base main` with piped stdin should succeed (no prompt)
- [ ] Integration test: existing local branch checkout still works without prompt
- [ ] Integration test: existing remote branch checkout still works without prompt

## Out of Scope

- Changing the `--base` flag name or making it positional
- Adding a prompt to `grov init` for base branch selection (it already has its own branch prompt)
- Adding interactive branch selection (e.g., fuzzy finder / list picker) — that's a separate feature
- Changing the base branch default for the `--base` flag itself (it will still use `default_branch()` when provided without a value — but `--base` requires a value, so this is N/A)
