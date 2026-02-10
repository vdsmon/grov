---
name: implement
description: Implement a task from an approved spec
---

Implement a specific task from a spec. The user will reference a spec file and optionally a task number.

## Process

1. **Read the spec**: Open the referenced spec file in `specs/`. Understand the full context — user story, acceptance criteria, technical design, and the specific task to implement.

2. **Approve if needed**: If the spec status is `draft`, present a summary of the spec (user story, acceptance criteria, tasks) and ask the user to confirm approval using AskUserQuestion. If the user approves, change the status to `in-progress`. If they decline, stop — do not proceed. If the spec is already `approved` or `in-progress`, skip this step.

3. **Create or switch to a feature branch**: All implementation work MUST happen on a feature branch, never on `main`. Use the naming convention `feat/<spec-name>` (e.g., `feat/001-dark-mode`). If the branch already exists from a previous task in the same spec, switch to it. If not, create it from `main`.

4. **Assess current state**: Before doing any work, determine what's already been done:
   - Check which tasks are already marked `[x]` in the spec — those are complete, skip them.
   - If no specific task number was requested, pick the first unchecked task (`[ ]`).
   - If a task appears unchecked but the feature branch already has related changes (e.g., the file modifications described in the task already exist), the task may have been partially completed in a previous session. In that case:
     - Read the affected files to understand what's already implemented.
     - Run `/ci-check` to see if the existing changes compile and pass tests.
     - Only implement the remaining parts of the task — do NOT redo work that's already done.
     - If the previous work is broken or incomplete in a way that's unclear, ask the user how to proceed.

5. **Update status**: If the spec status is `approved`, change it to `in-progress`.

6. **Implement the task**:
   - Follow the technical design in the spec
   - Match existing project conventions (see CLAUDE.md)
   - Only change files mentioned in the spec unless strictly necessary
   - Keep changes minimal and focused on the task

7. **Check the task off**: Mark the completed task as done in the spec (change `- [ ]` to `- [x]`).

8. **Run CI**: Use the `/ci-check` skill to verify formatting, linting, and tests pass.

9. **Check all spec checkboxes**: After CI passes, re-read the entire spec and check off ALL satisfied checkboxes — not just in the Tasks section, but also in Acceptance Criteria, Testing, and any other sections. Every `- [ ]` that is now true should become `- [x]`.

10. **If all tasks are done**: Update the spec status to `done`.

11. **Code review loop**: Use the Task tool with `subagent_type="feature-dev:code-reviewer"` to review all changes on the feature branch (diff against `main`).
    - If the reviewer finds actionable issues: append them as new tasks in the spec's Tasks section, then implement each one (steps 6–8), and re-run the code review. Repeat until the reviewer finds no actionable issues.
    - Use judgement to filter findings — skip suggestions that are out of scope, speculative, or contradict project conventions. Only act on clear bugs, logic errors, or security issues.

12. **Update CLAUDE.md**: Run the `/revise-claude-md` skill to capture any new patterns, conventions, or learnings from the implementation.

13. **Update README if relevant**: If the implementation changes CLI flags, commands, user-facing behavior, or other aspects documented in `README.md`, update the README to reflect the changes.

14. **Offer to commit and open a PR**: After all work is complete (or after each task if the user prefers incremental PRs), ask the user via AskUserQuestion whether they want to commit the changes and open a PR. If they accept, use the `/commit-push-pr` skill.

## Rules

- Implement ONE task at a time unless the user explicitly asks for more
- Do not refactor or "improve" code outside the task scope
- If the task is blocked or unclear, stop and ask rather than guessing
- If you discover a needed change not in the spec, note it but don't implement it — let the user decide whether to update the spec
