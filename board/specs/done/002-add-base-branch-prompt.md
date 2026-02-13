# Spec: Add base branch prompt to `grov add`

> Status: done

## User Story

As a developer, I want `grov add` to prompt me for the base branch when creating a new branch so that I can branch off the correct starting point instead of always getting the repo's default branch.

## Context

Currently, `grov add <branch>` silently uses the repo's default branch (e.g., `main`) as the base when creating a new branch that doesn't exist locally or remotely. The `--base` flag exists but is easy to forget. Since users typically want to branch off whatever they're currently working on, an interactive prompt with the current branch as the default is more ergonomic.

Source: `board/todo/add-base-branch-prompt.md` (consumed)

**Breaking change**: Non-TTY callers (scripts, CI) that relied on the silent `default_branch()` fallback will now get an error requiring `--base`. This is intentional — silent defaults are a footgun.

## Acceptance Criteria

- [x] When creating a new branch (not existing locally or remotely) without `--base`, `grov add` prompts: `? Base branch [<default>]:`
- [x] The prompt default is the current branch of the cwd's worktree (detected via `git -C <cwd> rev-parse --abbrev-ref HEAD`, using the original working directory — not the bare repo path)
- [x] If current branch detection fails (e.g., running from `repo.git` or project root), the prompt default falls back to `default_branch()`
- [x] If `--base` is provided, no prompt appears (existing behavior preserved)
- [x] If stdin is not a TTY and `--base` is not provided, the command exits non-zero with stderr containing: `--base is required when stdin is not a terminal` (note: anyhow context may prefix with `add failed:`)
- [x] The prompt uses the same styling as `grov init` prompts (cyan `?`, bold label, dim default in brackets)
- [x] Existing behavior for local and remote branch checkout is unchanged (no prompt in those paths — stderr must not contain the prompt marker `Base branch`)
- [x] README `grov add` section is updated to document the new prompt and `--base` requirement for non-TTY usage

## Technical Design

### Affected Files

- `src/commands/add.rs` — add interactive prompt logic for the new-branch path
- `src/commands/init.rs` — extract the `prompt()` helper so it can be shared
- `src/lib.rs` — add `mod ui;` for the shared prompt module
- `src/ui.rs` — new file: shared `prompt()` function
- `src/git/repo.rs` — add `current_branch()` helper
- `src/cli.rs` — no changes (the `--base` flag stays as-is)
- `README.md` — update `grov add` docs

### Approach

1. **Extract the `prompt()` function** from `src/commands/init.rs` into a new `src/ui.rs` module so both `init` and `add` can use it. Add `mod ui;` to `src/lib.rs`.

2. **Add current-branch detection** — a new helper `current_branch(cwd: &Path) -> Result<Option<String>>` in `src/git/repo.rs`. Must use `git -C <cwd> rev-parse --abbrev-ref HEAD` (passing cwd explicitly, not through the executor's `GIT_DIR` mechanism) to get the worktree's branch, not the bare repo HEAD. Before trusting the result, verify cwd is inside a work tree (`git -C <cwd> rev-parse --is-inside-work-tree`). Error contract: non-zero git exit or bare repo context → `Ok(None)`. Detached HEAD (`"HEAD"` literal) → `Ok(None)`. Spawn/IO failure → `Err`.

3. **Add TTY detection** — before prompting, check `std::io::stdin().is_terminal()` (available via `std::io::IsTerminal` in Rust 1.70+). If not a TTY and `--base` is absent, return an error.

4. **Add a `resolve_base_branch()` decision helper** in `src/commands/add.rs` — a pure function that takes `(base_flag: Option<&str>, current_branch: Option<&str>, is_tty: bool)` and returns `BaseBranchAction` where the action is `UseBase(String)`, `Prompt { default: Option<String> }`, or `ErrorNotTty`. The default branch is resolved lazily — only when the action requires it (i.e., not in the non-TTY error path) — to avoid triggering unrelated git errors before the intended `--base is required` message.

5. **Update the new-branch path in `add::execute()`** — when neither local nor remote branch exists and `--base` is `None`, call `resolve_base_branch()` and act on the result.

### Edge Cases

- **Detached HEAD**: `git rev-parse --abbrev-ref HEAD` returns `HEAD` literally. Treat this as "no current branch" and fall back to `default_branch()`.
- **Running from `repo.git` directory**: `--is-inside-work-tree` returns false for bare repos, so `current_branch()` returns `Ok(None)`. Fall back to `default_branch()`.
- **Running from project root** (not inside a worktree): Same as above — fall back.
- **`default_branch()` also fails**: The existing error propagation handles this (returns `GrovError`).
- **User enters empty string at prompt**: Use the displayed default (same pattern as `grov init`).
- **User enters a non-existent base branch**: Let git handle the error naturally when `add_worktree()` is called.

## Tasks

<!-- Each task should be small enough for one agent session -->

- [x] **Task 1**: Extract `prompt()` from `src/commands/init.rs` into `src/ui.rs`. Create `src/ui.rs` with the shared `prompt()` function. Update `init.rs` to import from `src/ui.rs`. Add `mod ui;` to `src/lib.rs`. Verify `cargo test` passes.
- [x] **Task 2**: Add `current_branch()` helper. Create `current_branch(cwd: &Path) -> Result<Option<String>>` in `src/git/repo.rs`. First check `git -C <cwd> rev-parse --is-inside-work-tree`, return `Ok(None)` if not. Then run `git -C <cwd> rev-parse --abbrev-ref HEAD`. Return `Ok(None)` for non-zero exit or detached HEAD (`"HEAD"` literal), `Err` for spawn/IO failures. Add unit tests: detached HEAD → `Ok(None)`, bare repo cwd → `Ok(None)`.
- [x] **Task 3**: Add `resolve_base_branch()` decision helper and interactive prompt to `grov add`. In `src/commands/add.rs`, add a pure `resolve_base_branch()` function and wire it into `execute()` for the new-branch path. Use the shared `prompt()` from `src/ui.rs`. Add unit tests for the decision helper (all input combinations).
- [x] **Task 4**: Add integration test — non-TTY error case. `grov add new-branch` with piped stdin and no `--base` should exit non-zero with `--base is required` on stderr.
- [x] **Task 5**: Add integration test — `--base` bypass. `grov add new-branch --base main` with piped stdin should succeed without prompting.
- [x] **Task 6**: Add integration tests — existing branch paths unchanged. Verify local branch checkout and remote branch checkout succeed and stderr does not contain the prompt marker `Base branch`.
- [x] **Task 7**: Update README `grov add` section to document the new base-branch prompt and `--base` requirement for non-TTY usage.

## Testing

- [x] Unit test for `current_branch()` — detached HEAD returns `Ok(None)`
- [x] Unit test for `current_branch()` — bare repo cwd returns `Ok(None)`
- [x] Unit tests for `resolve_base_branch()` — all combinations: base provided, TTY with current branch, TTY without current branch (falls back), non-TTY error
- [x] Integration test: `grov add new-branch` with piped stdin and no `--base` exits non-zero with stderr containing `--base is required`
- [x] Integration test: `grov add new-branch --base main` with piped stdin succeeds (no prompt)
- [x] Integration test: existing local branch checkout works and stderr does not contain `Base branch`
- [x] Integration test: existing remote branch checkout works and stderr does not contain `Base branch`

## Out of Scope

- Changing the `--base` flag name or making it positional
- Adding a prompt to `grov init` for base branch selection (it already has its own branch prompt)
- Adding interactive branch selection (e.g., fuzzy finder / list picker) — that's a separate feature
- Backwards compatibility for non-TTY callers — they must now pass `--base`
