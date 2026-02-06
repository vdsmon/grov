# Test Coverage Analysis

## Current State

The codebase has **11 integration tests** (across 4 test files) and **11 unit tests** (in `paths.rs` and `worktree.rs`). Two integration tests (`cli_add`) are currently failing due to a test infrastructure issue in `create_bare_repo`.

### Coverage by Module

| Module | Functions | Tested | Notes |
|--------|-----------|--------|-------|
| `src/paths.rs` | 3 | 3 (unit) | Excellent coverage |
| `src/commands/remove.rs` | 1 | 1 (4 integration tests) | Best-tested command |
| `src/commands/init.rs` | 1 | 1 (2 integration tests) | Core flow covered |
| `src/commands/add.rs` | 1 | partial (2 integration tests) | Missing key branch |
| `src/commands/list.rs` | 2 | partial (2 integration tests) | Output content not verified |
| `src/git/worktree.rs` | 8 | 1 (unit) | 7 functions untested directly |
| `src/git/repo.rs` | 3 | 0 | Implicitly exercised only |
| `src/git/status.rs` | 2 | 0 | Implicitly exercised only |
| `src/git/executor.rs` | 2 | 0 | Implicitly exercised only |
| `src/config.rs` | 2 | 0 | Implicitly exercised only |
| `src/errors.rs` | — | 0 | Structural definitions only |
| `src/commands/completions.rs` | 1 | 0 | Low priority |

---

## Recommended Improvements (Priority Order)

### 1. Fix the Failing `cli_add` Tests (Critical)

The two tests in `tests/cli_add.rs` are failing. The `create_bare_repo` helper panics during test setup. This needs to be diagnosed and fixed before any new test work — broken tests undermine confidence in the entire suite.

### 2. Add `find_worktree` Unit Tests (High Priority)

`find_worktree` in `src/git/worktree.rs:117` has two matching strategies (branch name vs directory name) but no direct tests. This is pure logic with no git dependency — it operates on a `&[WorktreeInfo]` slice — so it's trivially unit-testable.

**Suggested tests:**
- Match by exact branch name
- Match by directory name (last path component)
- No match returns `None`
- Branch match takes precedence when both could match
- Worktree with no branch (detached HEAD) matches only by directory name

### 3. Add `list` Output Verification Tests (High Priority)

The current list tests (`tests/cli_list.rs`) only check that output *contains* "main". They don't verify:

- **`[clean]` / `[dirty]` status markers** — the dirty indicator is a key feature of `list` and is completely unverified
- **Current worktree `*` marker** — running `list` from within a worktree should mark it
- **Detached HEAD display** — `(detached)` fallback text
- **`format_ahead_behind` output** — the `↑N ↓N` formatting logic in `src/commands/list.rs:67` is pure and easily unit-testable

**Suggested tests:**
- Unit test for `format_ahead_behind`: `None` → empty, `Some((0,0))` → empty, `Some((3,0))` → `↑3`, `Some((0,2))` → `↓2`, `Some((1,4))` → `↑1 ↓4`
- Integration test: create a worktree, write an uncommitted file, verify `[dirty]` appears in output
- Integration test: verify `[clean]` appears for a worktree with no changes

### 4. Test the `add --base` Flag and Remote Branch Tracking (High Priority)

`src/commands/add.rs` has three code paths (lines 30-48):
1. Local branch exists → checked out (tested)
2. Remote branch exists → tracked with `--track -b` (NOT tested)
3. New branch from base (tested, but `--base` flag NOT tested)

The remote-tracking path (path 2) is the most complex and the one most likely to have edge-case bugs. The `--base` flag in path 3 is also never exercised.

**Suggested tests:**
- `add` with `--base other-branch` creates worktree from specified base
- `add` with a branch that exists on origin but not locally creates a tracking branch

### 5. Add `config.rs` Unit Tests (Medium Priority)

`read_config` and `write_config` in `src/config.rs` handle TOML serialization. Currently only exercised indirectly through `init`. Key untested behaviors:

- `read_config` returns `Default` when file doesn't exist
- `read_config` returns `Default` when file contains invalid TOML (the `unwrap_or_default` on line 24)
- Round-trip: `write_config` then `read_config` preserves values
- Config with empty prefix serializes/deserializes correctly

These are pure filesystem + TOML operations and straightforward to test with `tempfile`.

### 6. Add `find_bare_repo` Discovery Tests (Medium Priority)

`find_bare_repo` in `src/git/repo.rs:19` implements a 4-step discovery strategy:
1. Direct bare repo check
2. `repo.git` child directory
3. `git rev-parse --git-common-dir` (from within worktree)
4. Walk up parent directories

Only strategies 2 and 3 are exercised by integration tests (indirectly). Strategy 1 (passing a bare repo path directly) and strategy 4 (deeply nested directory) are not tested.

**Suggested tests:**
- Start from the bare repo itself → should return it directly
- Start from a subdirectory several levels below the project → should walk up and find `repo.git`
- Start from a completely unrelated directory → should return `BareRepoNotFound` error
- Start from within a worktree → should find the bare repo via `git-common-dir`

### 7. Add `ahead_behind` Parsing Unit Test (Medium Priority)

`ahead_behind` in `src/git/status.rs:15` parses tab-separated output from `git rev-list`. The parsing logic (lines 31-38) silently swallows parse failures via `unwrap_or(0)`, and handles the "no upstream" case by returning `None` on any error (line 40). This error-swallowing behavior should be explicitly tested.

Since this function calls git directly, testing the parsing logic requires either:
- An integration test with a repo that has an upstream configured (set up a remote tracking branch in the test helper)
- Extracting the parsing into a separate pure function that can be unit-tested

### 8. Test Error Messages and Exit Codes (Low Priority)

The following error scenarios are tested only for `remove`:
- Dirty worktree check → tested in `cli_remove`
- Worktree not found → NOT tested

Untested error scenarios across commands:
- `add` when worktree directory already exists (line 25 in `add.rs`)
- `add` when branch name doesn't exist locally or remotely and no base is given and `default_branch` fails
- `remove` targeting the bare repo entry (should be prevented, `remove.rs` line 20-ish)
- Running any command outside a grov project (no bare repo found)

### 9. Test `worktree_dir` Edge Cases (Low Priority)

`worktree_dir` has 2 unit tests but doesn't cover:
- Branch names with multiple slashes (`feature/auth/login` → `feature-auth-login`)
- Branch names that sanitize to collisions (e.g., `a/b` and `a-b` both produce `a-b`)
- The interaction between `sanitize_branch_name` and `worktree_dir` for unusual inputs

---

## Structural Observations

**Test helper duplication**: `tests/cli_init.rs` has its own `run()` helper (line 94) that duplicates `tests/common/mod.rs::run()`. The init tests don't use the shared `common` module at all. Consider unifying.

**No negative/error path tests for `init`**: The `init` command has no tests for failure cases (invalid URL, directory already exists, clone failure).

**No test for `--path` flag**: Both `init --path` and `add --path` accept custom paths but neither is tested.

**`parse_porcelain_output` test duplicates implementation**: The unit test in `worktree.rs:140-200` copy-pastes the parsing logic from `list_worktrees` rather than calling the function. This means the test doesn't actually verify the function — it verifies a copy of the logic. This test should be refactored to either call `list_worktrees` with a mock or extract the parser into a standalone function.
