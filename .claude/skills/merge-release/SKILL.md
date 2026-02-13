---
name: merge-release
description: Merge current PR into main, then optionally release
---

Merge the current branch's PR into main and optionally run a release.

## Steps

### 1. Identify the PR

Run `gh pr view --json number,title,state,headRefName` for the current branch. If no PR exists or it's already merged, inform the user and stop.

### 2. Check for uncommitted changes

Run `git status`. If there are staged, unstaged, or untracked changes:

- **Do not commit or stash automatically.**
- Show the user what's dirty/untracked.
- Suggest options using AskUserQuestion:
  - "Commit to feature branch and push" — use `/commit` skill, then `git push`
  - "Stash changes" — run `git stash --include-untracked`
  - "Add to .gitignore" — for files that should never be tracked (e.g. scratch files, local tooling dirs)
  - "I'll handle it manually" — stop and let the user take over
- After the user's choice is resolved, re-check that the working tree is clean enough to proceed.

### 3. Ensure branch is pushed

Check if the local branch is ahead of the remote (`git status -sb`). If ahead, push.

### 4. Wait for CI

Run `gh pr checks --watch --fail-on-failure`. This blocks until all checks complete.

- If checks pass, continue.
- If checks fail, show the failure details and stop. Do not attempt to merge.

### 5. Merge the PR

Run `gh pr merge --squash --delete-branch`. The repo requires linear history, so squash merge is the default.

### 6. Switch to main and pull

```sh
git checkout main
git pull
```

### 7. Ask about release

Ask the user using AskUserQuestion whether they want to release now or stop:

- "Yes, release" — run the `/release` skill
- "No, stop here" — done

If the user chooses to release, invoke the `/release` skill which handles clean-tree verification, CI, bump level prompt, and `cargo release`.
