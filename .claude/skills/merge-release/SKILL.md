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

Run `gh pr checks --watch`. This blocks until all checks complete.

- If checks pass, continue.
- If checks fail, show the failure details and stop. Do not attempt to merge.

### 5. Review and resolve CodeRabbit comments

CodeRabbit reviews PRs automatically. Its comments must be addressed before merging.

1. Fetch PR review comments: `gh api repos/{owner}/{repo}/pulls/{number}/comments`
2. Fetch unresolved review threads via GraphQL:
   ```
   gh api graphql -f query='{ repository(owner: "{owner}", name: "{repo}") { pullRequest(number: {number}) { reviewThreads(first: 50) { nodes { id isResolved comments(first: 1) { nodes { body author { login } path line } } } } } } }'
   ```
3. For each unresolved thread:
   - Show the comment to the user (file, line, summary of the issue).
   - If already fixed (e.g. by a subsequent commit), resolve it:
     ```
     gh api graphql -f query='mutation { resolveReviewThread(input: { threadId: "{id}" }) { thread { isResolved } } }'
     ```
   - If not yet fixed, fix the issue, commit, push, and then resolve the thread.
   - If the comment is not applicable, ask the user whether to resolve it anyway.
4. Confirm all threads are resolved before proceeding.

### 6. Merge the PR

Run `gh pr merge --squash --delete-branch`. The repo requires linear history, so squash merge is the default.

**Important**: Ensure the squash merge title uses conventional commit format (e.g., `feat: Add dark mode`, `fix: Handle empty input`). GitHub defaults to the PR title — verify it's conventional before merging.

### 7. Switch to main and pull

```sh
git checkout main
git pull
```

### 8. Check for release-please PR

Releases are automated via release-please. After merging to main, check if a Release PR exists:

```sh
gh pr list --label 'autorelease: pending'
```

- If a Release PR exists, show it to the user (`gh pr view <number>`) and ask via AskUserQuestion:
  - "Merge the Release PR now" — merge it with `gh pr merge --squash`
  - "Skip for now" — done
- If no Release PR exists, inform the user that release-please will create one on the next push to main (if conventional commits warrant a version bump).
