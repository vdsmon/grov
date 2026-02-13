---
name: implement
description: Implement a task from an approved spec
---

Implement a specific task from a spec. The user will reference a spec file and optionally a task number. Each invocation handles ONE task.

## Process

1. **Pick or receive the spec**:
   - If the user provided arguments (a spec file path or number), use that directly — skip the picker.
   - If no arguments were provided, scan `board/specs/active/` for `.md` files.
     - If there are specs: present a picker using AskUserQuestion. Show up to 5 files (most recent first by filename). Each option label is the spec filename (without `.md`), and the description is the `# Spec: Title` and `Status` from the file.
     - If there are no specs: tell the user there are no specs to implement and stop.

2. **Read the spec**: Open the selected spec file in `board/specs/active/`. Understand the full context — user story, acceptance criteria, technical design, and the specific task to implement.

3. **Ensure spec is in-progress**: Handle the spec status:
   - `draft` → present a summary (user story, acceptance criteria, tasks) and ask the user to confirm approval via AskUserQuestion. If approved, change status to `in-progress`. If declined, stop.
   - `approved` → change status to `in-progress`.
   - `in-progress` → no change needed.

4. **Create or switch to a feature branch**: All implementation work MUST happen on a feature branch, never on `main`. Use the naming convention `feat/<spec-name>` (e.g., `feat/001-dark-mode`). If the branch already exists from a previous task in the same spec, switch to it. If not, create it from `main`.

5. **Pick the task**: Determine which task to work on:
   - Check which tasks are already marked `[x]` in the spec — those are complete, skip them.
   - If a specific task number was requested, use that.
   - Otherwise, pick the first unchecked task (`[ ]`).
   - If the task appears unchecked but the feature branch already has related changes, it may have been partially completed. Read the affected files and only implement remaining parts. If the state is unclear, ask the user.
   - **Determine if this is the last task** — count remaining unchecked tasks. This affects the plan template.

6. **Plan the task** (mandatory before writing any code):
   - Use `EnterPlanMode` to transition into plan mode.
   - Read every file listed in the task and the spec's "Affected Files" section.
   - Explore the existing code to understand current patterns, function signatures, imports, and test structure.
   - Write the plan using the template below. **The plan is the only thing that reliably survives context loss after plan mode, so it must be self-contained.**
   - Use `ExitPlanMode` to present the plan for user approval.
   - Do NOT write any code until the plan is approved.

7. **Implement**: Follow the approved plan (including all wrap-up steps in the plan).

## Plan Template

Every plan MUST follow this structure. Fill in each section.

```markdown
# Plan: Task N — <short description>

## Context

Spec: `board/specs/active/NNN-name.md`
Task N of M (is_last_task: yes/no)
Branch: `feat/NNN-name`

## Changes

### 1. <file or component>
<what to do: functions/structs to add or change, with signatures>

### 2. <file or component>
...

### Order of changes
<what depends on what>

### Deviations from spec
<any deviations from the spec's technical design, with justification — or "None">

## Verification

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Wrap-up (do these after implementation + verification)

1. In `board/specs/active/NNN-name.md`, check off Task N (`- [ ]` → `- [x]`)
2. Re-read the full spec — check off any newly satisfied boxes in Acceptance Criteria and Testing sections
3. Ask the user: continue to next task (`/implement`), commit (`/commit-push-pr`), or stop?

<!-- If is_last_task: yes, ALSO include these steps: -->
4. Run code review: use Task tool with `subagent_type="feature-dev:code-reviewer"` to review all changes on the branch (diff against `main`). Only act on clear bugs, logic errors, or security issues — skip out-of-scope or speculative suggestions.
5. Run `/claude-md-management:revise-claude-md` to capture new patterns or conventions
6. If the implementation changes CLI behavior or flags, update `README.md`
7. Move spec from `board/specs/active/` to `board/specs/done/` and set status to `done`
8. Ask the user whether to commit and open a PR (`/commit-push-pr`). Use a conventional commit title. The spec move must be included in the commit.
```

## Rules

- Implement ONE task per invocation — do not chain tasks in one session
- Do not refactor or "improve" code outside the task scope
- If the task is blocked or unclear, stop and ask rather than guessing
- If you discover a needed change not in the spec, note it but don't implement it — let the user decide whether to update the spec
