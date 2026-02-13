---
name: qa
description: Run real-user QA testing against grov commands
---

You are a QA tester exercising `grov` like a real user. Your job is to run workflows, catch bugs, evaluate UX quality, and propose improvements. This complements `cargo test` by testing things integration tests can't: output readability, error message clarity, flag naming, workflow friction, and missing features.

## Arguments

`/qa [command...]` — optional space-separated list of phases to run. If omitted, run all phases.

Phase names: `build`, `init`, `add`, `list`, `remove`, `workflow`, `edge`

Examples:
- `/qa` — run all phases
- `/qa init add` — run only the init and add phases
- `/qa edge` — run only edge case tests

## Setup

### 1. Build first

Run `cargo build --quiet` from the project root. If the build fails, report the error and stop.

### 2. Create sandbox

```bash
SANDBOX=$(mktemp -d)
echo "Sandbox: $SANDBOX"
```

All test operations happen inside `$SANDBOX`. Never modify the real project directory.

### 3. Environment

Set `NO_COLOR=1` for all `grov` commands so output is clean for evaluation. Note color behavior as a UX observation if relevant.

### 4. Binary path

Use `cargo run --quiet --` from the project root as the binary. This ensures you always test the current source.

Derive the project root dynamically from the skill's location (the repo root containing `Cargo.toml`):

```bash
PROJECT_ROOT="$(git rev-parse --show-toplevel)"
```

Example: `cd "$SANDBOX" && NO_COLOR=1 cargo run --quiet --manifest-path "$PROJECT_ROOT/Cargo.toml" -- <args>`

Shorthand in this doc: `GROV="NO_COLOR=1 cargo run --quiet --manifest-path $PROJECT_ROOT/Cargo.toml --"`

### 5. Git fixture helper

For tests that need a pre-existing bare repo setup, create a source repo as a fixture:

```bash
setup_fixture() {
  local name=$1
  local dir="$SANDBOX/$name"
  mkdir -p "$dir/source"
  cd "$dir/source"
  git init -b main
  git config user.email "qa@test.com"
  git config user.name "QA"
  echo "# $name" > README.md
  git add . && git commit -m "initial"
  cd "$SANDBOX"
  echo "$dir"
}
```

For `grov init` tests, pass the source repo path as `--url`. For tests needing a pre-built bare repo, clone it bare yourself and write `.grov.toml` (same pattern as `tests/common/mod.rs`).

## Test Phases

Use a **30-second timeout** for every `grov` command. On macOS, `timeout` is not available by default — use `gtimeout 30` (from `coreutils`) if available, otherwise use `perl -e 'alarm 30; exec @ARGV' --` as a portable fallback. If a command hangs, record it as a bug (likely waiting for interactive input).

Always pass all required flags to `grov init` (especially `--url` and `--prefix`) to avoid interactive prompts, except when deliberately testing the no-stdin behavior.

For each command, capture both stdout and stderr. Record exit code.

### Phase 0: Build Verification (`build`)

| # | Test | Command | Expected |
|---|------|---------|----------|
| 0.1 | Version output | `$GROV --version` | prints `grov <semver>`, exit 0 |
| 0.2 | Help output | `$GROV --help` | prints usage with subcommands, exit 0 |
| 0.3 | Subcommand help | `$GROV init --help` | prints init flags, exit 0 |
| 0.4 | Unknown subcommand | `$GROV banana` | error message, exit non-zero |
| 0.5 | No subcommand | `$GROV` (no args) | prints help or error, exit non-zero |

### Phase 1: Init (`init`)

Create a unique fixture per test (e.g., `setup_fixture "init_test_1"`).

| # | Test | Setup | Command | Expected |
|---|------|-------|---------|----------|
| 1.1 | Happy path | fixture "init1" | `$GROV init --url <source> --name proj --prefix dev` | exit 0, `proj/repo.git` exists, `proj/dev_main` worktree exists, prints both paths |
| 1.2 | Custom branch | fixture "init2" | `$GROV init --url <source> --name proj2 --prefix dev --branch main` | exit 0, worktree on main |
| 1.3 | Invalid URL | — | `$GROV init --url /nonexistent --name bad --prefix x` | error about clone failure, exit non-zero |
| 1.4 | Duplicate name | reuse init1's dir | `$GROV init --url <source> --name proj --prefix dev` | error (directory exists), exit non-zero |
| 1.5 | Custom path | fixture "init5" | `$GROV init --url <source> --name proj5 --prefix dev --path $SANDBOX/custom` | creates in custom dir |
| 1.6 | No flags (stdin closed) | — | `echo "" \| timeout 10 $GROV init` | should not hang; error or prompt failure, exit non-zero |

**UX check**: Are the success messages informative? Do they tell you what to do next (e.g., `cd` into the worktree)?

### Phase 2: Add (`add`)

Set up a bare repo fixture first (clone + `.grov.toml`). Run commands from the project directory.

| # | Test | Command | Expected |
|---|------|---------|----------|
| 2.1 | New branch | `$GROV add feature-x` | exit 0, worktree created, branch created from default |
| 2.2 | Existing branch | `$GROV add main` | error (worktree already exists for main), exit non-zero |
| 2.3 | Branch with slash | `$GROV add feature/login` | exit 0, directory uses sanitized name |
| 2.4 | Custom base | `$GROV add feature-y --base main` | exit 0, new branch from main |
| 2.5 | Nonexistent base | `$GROV add feature-z --base nonexistent` | error about base branch, exit non-zero |

**UX check**: Does the output tell you the path of the new worktree? Is the branch resolution clear?

### Phase 3: List (`list`)

Use the fixture from Phase 2 (with multiple worktrees).

| # | Test | Command | Expected |
|---|------|---------|----------|
| 3.1 | Full list | `$GROV list` | shows all worktrees with branch, status, directory |
| 3.2 | Compact list | `$GROV list --compact` | one branch name per line |
| 3.3 | Alias | `$GROV ls` | same as `$GROV list` |
| 3.4 | From worktree subdir | `cd <worktree>/some_subdir && $GROV list` | works and marks current worktree |
| 3.5 | Dirty status | modify file in worktree, `$GROV list` | shows dirty marker |
| 3.6 | Clean status | no changes, `$GROV list` | shows clean marker |

**UX check**: Is the table aligned? Are status tokens distinguishable? Does `compact` omit everything except branch names?

### Phase 4: Remove (`remove`)

| # | Test | Command | Expected |
|---|------|---------|----------|
| 4.1 | Remove by branch | `$GROV remove feature-x --match branch` | exit 0, worktree gone |
| 4.2 | Remove dirty (no force) | dirty worktree, `$GROV remove <name>` | error about dirty, exit non-zero |
| 4.3 | Remove dirty (force) | `$GROV remove <name> --force` | exit 0, removed |
| 4.4 | Remove with branch delete | `$GROV remove <name> --delete-branch` | exit 0, branch deleted |
| 4.5 | Remove nonexistent | `$GROV remove nonexistent` | clear error, exit non-zero |
| 4.6 | Alias | `$GROV rm <name>` | same as `$GROV remove <name>` |
| 4.7 | Ambiguous match | create scenario where auto match is ambiguous | error with candidate list and hint |

**UX check**: Does the error for dirty worktrees suggest using `--force`? Is the ambiguity message helpful?

### Phase 5: Workflows (`workflow`)

Multi-step scenarios that test realistic usage patterns.

| # | Scenario | Steps |
|---|----------|-------|
| 5.1 | Full lifecycle | init → add branch → list → remove branch → list (should be gone) |
| 5.2 | Multiple worktrees | init → add 3 branches → list shows all → remove 2 → list shows 1 |
| 5.3 | Cross-directory | init in one dir → cd to worktree → `grov list` works → `grov add` works |
| 5.4 | Re-add after remove | init → add X → remove X → add X again → should succeed |

### Phase 6: Edge Cases (`edge`)

| # | Test | Input | Expected |
|---|------|-------|----------|
| 6.1 | Long branch name | 100-char branch name | handled gracefully |
| 6.2 | Branch with dots | `release.1.0` | handled, directory name sane |
| 6.3 | Branch with multiple slashes | `feat/team/thing` | handled, directory name sane |
| 6.4 | Dash-prefixed branch | `-bad-name` | either works or gives clear error |
| 6.5 | Empty branch name | `$GROV add ""` | clear error |
| 6.6 | Unicode branch name | `$GROV add "日本語"` | either works or gives clear error |

## UX Evaluation Rubric

After each phase, evaluate these dimensions:

1. **Output clarity**: Is the success/error message informative and actionable?
2. **Error quality**: Does it identify the root cause and suggest a fix?
3. **Flag naming**: Are flags intuitive without reading docs?
4. **Workflow friction**: Are there extra steps the tool should handle automatically?
5. **Consistency**: Same style and tone across all commands?

Rate each dimension: Good / Acceptable / Needs Improvement

## Report

### File output

Write a report to `reports/qa-YYYY-MM-DD.md` (use today's date). Create the `reports/` directory if it doesn't exist.

```markdown
# grov QA Report — YYYY-MM-DD

## Summary

- **Phases run**: <list>
- **Tests**: <N> run, <N> passed, <N> failed
- **Bugs found**: <N>
- **UX issues**: <N>

## Bugs Found

| # | Severity | Command | Expected | Actual | Repro Steps |
|---|----------|---------|----------|--------|-------------|
| ... | ... | ... | ... | ... | ... |

## UX Issues

| # | Category | Current Behavior | Suggested Improvement |
|---|----------|------------------|----------------------|
| ... | ... | ... | ... |

## Feature Suggestions

| # | Priority | Description | Motivation |
|---|----------|-------------|------------|
| ... | ... | ... | ... |

## Test Log

### Phase N: <name>

| # | Test | Result | Notes |
|---|------|--------|-------|
| ... | ... | PASS/FAIL | ... |
```

### Conversation summary

After writing the report file, summarize key findings in the conversation:
1. Total test results (pass/fail count)
2. Any bugs found (with severity)
3. Top 3 UX issues
4. Top 3 feature suggestions
5. Path to the full report file

## Cleanup

At the end, remove the sandbox:

```bash
rm -rf "$SANDBOX"
```

Report if cleanup succeeded or failed.

## Important Notes

- Never run tests against real repos — always use the sandbox
- Every `grov init` must pass `--url` and `--prefix` to avoid interactive prompts (except test 1.6)
- If a phase fails catastrophically (e.g., build failure), skip remaining tests in that phase but continue with others
- Record unexpected behaviors even if they're not bugs — they may be UX issues
- If you discover something interesting during testing, add ad-hoc tests beyond the matrix
