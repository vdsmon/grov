---
name: release
description: Understand and manage release-please automated releases
disable-model-invocation: true
---

Releases are automated via release-please. There is no manual release command.

## How it works

1. Merge PRs into `main` using **conventional commit** titles (squash merge)
2. release-please maintains a "Release PR" that tracks pending changes
3. When ready, merge the Release PR → release-please creates a git tag + GitHub Release
4. The tag triggers crates.io publish and binary builds automatically

## Commit type → version bump (0.x)

- `fix:` → patch (0.3.1 → 0.3.2)
- `feat:` → minor (0.3.1 → 0.4.0)
- `feat!:` or `BREAKING CHANGE` → minor (0.x treats breaking as minor)
- `chore:`, `docs:`, `ci:`, `refactor:`, `test:` → no version bump (hidden in changelog)

## Checking release status

```sh
# See if a Release PR exists
gh pr list --label 'autorelease: pending'

# View the Release PR
gh pr view <number>
```

## Merging a release

1. Ensure CI passes on the Release PR
2. Review the CHANGELOG preview in the PR body
3. Merge the Release PR (use merge commit, not squash — release-please manages its own commit)
4. Verify: tag created, GitHub Release published, crates.io publish succeeded, binaries attached

## Troubleshooting

- **No Release PR appears**: Ensure commits since last release use conventional format. Non-conventional commits are ignored.
- **Wrong version**: Check `release-please-config.json` for `release-as` overrides or bump settings.
- **CI doesn't run on Release PR**: The `RELEASE_PLEASE_TOKEN` PAT may have expired. Regenerate and update the repo secret.
