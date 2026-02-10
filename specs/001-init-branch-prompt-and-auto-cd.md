# Spec: Init branch prompt and print paths

> Status: done

## User Story

As a developer, I want `grov init` to ask me which branch to check out (with the detected default as the suggestion) and then print the paths to both the bare repo and the worktree, so I know where everything is and can quickly cd into the right place.

## Context

Currently `grov init` auto-detects the default branch via `git symbolic-ref refs/remotes/origin/HEAD` with no option to override, and only prints a single-line success message. Two improvements:

1. Let the user choose/override the initial branch (some repos use `master`, `develop`, etc.)
2. After init, clearly print both the `repo.git` path and the worktree path so the user can cd into either

## Acceptance Criteria

- [x] `grov init` prompts for the initial branch name after the prefix prompt
- [x] The prompt shows the auto-detected default branch as the default value (falls back to "main" if detection fails)
- [x] A `--branch` CLI flag skips the prompt (consistent with `--url`, `--name`, `--prefix`)
- [x] After successful init, the success message prints both the `repo.git` path and the worktree path
- [x] Non-interactive usage (`--url --name --prefix --branch`) works without any prompts
- [x] Existing integration tests continue to pass

## Technical Design

### Affected Files

- `src/cli.rs` — add `--branch` flag to `Init` variant
- `src/commands/init.rs` — add `branch` parameter to `execute()`; add branch prompt step after fetch; use chosen branch for `add_worktree`; update success message to print both paths
- `src/lib.rs` — thread `branch` from CLI to `init::execute()`

### Approach

**Branch prompt**:
- After the fetch step (line 105 of current `init.rs`), detect the default branch using existing `default_branch()`. If it fails, fall back to `"main"`.
- Prompt: `"Default branch [<detected>]: "` using the existing `prompt()` helper.
- If `--branch` flag was provided, skip the prompt entirely.
- Use the chosen branch instead of the auto-detected one for the `add_worktree` call.

**Print paths**:
- Update the success message at the end of `init::execute()` to print both the bare repo path and the worktree path on separate lines, e.g.:
  ```
  ✓ Initialized myproject/

    bare repo   myproject/repo.git
    worktree    myproject/wt_main
  ```

### Edge Cases

- `default_branch()` fails (e.g., `refs/remotes/origin/HEAD` not set) — fall back to "main" as prompt default
- User provides a branch that doesn't exist on remote — `add_worktree` already handles this (creates from HEAD)
- User provides `--branch` with a non-existent branch — same behavior, consistent with `--url`/`--name`/`--prefix`
- Piped/non-interactive usage — `--branch` flag bypasses the prompt; if stdin is not a TTY and no `--branch`, the prompt reads empty and uses the default

## Tasks

<!-- Each task should be small enough for one agent session -->

- [x] **Task 1**: Add `--branch` flag to `Init` in `src/cli.rs` and thread it through `src/lib.rs` to `init::execute()` as a new parameter
- [x] **Task 2**: Add the branch prompt to `src/commands/init.rs` — after fetch, detect default branch (with "main" fallback), prompt if `--branch` not provided, use the result for `add_worktree`
- [x] **Task 3**: Update the success message in `src/commands/init.rs` to print both the `repo.git` path and the worktree path
- [x] **Task 4**: Add/update integration tests in `tests/cli_init.rs` — test `--branch` flag, test that output contains both paths

## Testing

- [x] Unit tests: none needed (no new pure functions)
- [x] Integration tests in `tests/cli_init.rs`:
  - `--branch main` creates worktree for `main`
  - `--branch` with a custom branch name works
  - success output contains both paths
  - existing tests still pass

## Out of Scope

- Shell completions for the `--branch` flag (clap handles this automatically)
- Shell wrapper functions for auto-cd
- Applying similar changes to `grov add`
