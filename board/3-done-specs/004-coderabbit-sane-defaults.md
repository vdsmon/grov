# Spec: CodeRabbit Sane Defaults

> Status: done

## User Story

As a maintainer, I want CodeRabbit configured with minimal noise so that PR reviews are fast, focused on real code issues, and don't clutter the conversation with poems, diagrams, or suggestions on non-code files.

## Context

The current `.coderabbit.yaml` is minimal — it only skips auto-review on release-please PRs. CodeRabbit is slow on simple PRs and generates noise (poems, sequence diagrams, suggested reviewers/labels, related PR links) that isn't useful for a small single-maintainer Rust CLI project. The config needs to be tightened to scope reviews to code files only and disable extraneous features.

## Acceptance Criteria

- [ ] CodeRabbit only reviews files in `src/`, `tests/`, and `Cargo.toml` — all other paths are excluded
- [ ] The poem is disabled
- [ ] Sequence diagrams are disabled
- [ ] Suggested reviewers are disabled
- [ ] Suggested labels are disabled
- [ ] Related PRs/issues are disabled
- [ ] Code review effort estimate is disabled
- [ ] Review profile is set to "chill"
- [ ] Rust-specific path instructions are set for `*.rs` files (focus on correctness and error handling)
- [ ] Auto-review still skips release-please PRs (`autorelease: pending` label)
- [ ] The walkthrough is collapsed by default

## Technical Design

### Affected Files

- `.coderabbit.yaml` — complete rewrite with expanded configuration

### Approach

Replace the current 5-line config with a comprehensive config that:

1. Sets `reviews.profile` to `"chill"`
2. Disables decorative features: `poem: false`, `sequence_diagrams: false`
3. Disables suggestion features: `suggested_labels: false`, `suggested_reviewers: false`, `related_issues: false`, `related_prs: false`, `estimate_code_review_effort: false`
4. Collapses walkthrough: `collapse_walkthrough: true`
5. Scopes auto-review via `path_filters` to only include `src/**`, `tests/**`, and `Cargo.toml`
6. Preserves the existing `ignore_labels: ["autorelease: pending"]` behavior
7. Adds `path_instructions` for `**/*.rs` with guidance: "Focus on correctness and error handling."
8. Disables chat art: `chat.art: false`

### Edge Cases

- **New top-level directories**: If new Rust code directories are added outside `src/` and `tests/`, they won't be reviewed unless the path filter is updated. This is acceptable — the config can be amended when needed.
- **Cargo.toml changes in subdirectories**: The filter `Cargo.toml` matches root only. If workspace members are added later, the filter would need `**/Cargo.toml`. For now, root-only is correct since this is a single-crate project.

## Tasks

- [x] **Task 1**: Replace `.coderabbit.yaml` with the full configuration described in the approach section. Validate YAML syntax.

## Testing

- [ ] Manual: open a PR touching only `src/` files and verify CodeRabbit reviews it without poem, diagrams, or suggestions
- [ ] Manual: open a PR touching only `.github/` or `board/` files and verify CodeRabbit does not review it
- [ ] Manual: verify release-please PRs are still skipped

## Out of Scope

- CodeRabbit knowledge base or learnings configuration
- Integration with Jira/Linear
- Custom pre-merge checks
- Tone customization beyond profile selection
