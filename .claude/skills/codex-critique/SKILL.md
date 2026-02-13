---
name: codex-critique
description: Send a spec or plan to Codex CLI for an independent critique
---

Get an independent critique from Codex CLI on a spec, plan, or document. Follow this process:

1. **Identify the target**: The user may provide a file path as an argument (e.g., `board/specs/active/003-feature.md`). If no argument is given, scan `board/specs/active/` for draft or approved specs and present a picker using AskUserQuestion. If there are no specs, ask the user what they want critiqued.

2. **Read the file**: Read the target file so you understand what's being sent for critique.

3. **Send to Codex**: Run Codex in non-interactive mode to critique the document:

   ```
   codex exec --json --full-auto "You are reviewing a spec/plan for a Rust CLI project called grov (a git worktree manager). Read the file at <FILE_PATH> and critique it. Focus on: missing edge cases, testability gaps, ambiguous requirements, scope creep, tasks that are too large for one session, missing acceptance criteria, and whether the technical approach fits the existing codebase patterns (check CLAUDE.md and src/ for context). Be concise and actionable. Structure your response as a numbered list of issues. If the spec is solid, say so briefly and note any minor suggestions." 2>&1
   ```

   Replace `<FILE_PATH>` with the actual path.

4. **Parse the response**: Extract all `agent_message` items from the JSONL output. Ignore `reasoning`, `command_execution`, and metadata lines.

5. **Present the critique**: Show Codex's feedback to the user clearly, prefixed with "**Codex Critique:**". Include all substantive feedback.

6. **Offer to revise**: After presenting the critique, ask the user if they want you to revise the spec based on any of the feedback. If yes, make the edits. If you disagree with any of Codex's points, say so and explain why.

## Multi-turn follow-up

- After the first Codex call, save the session ID from the `thread.started` JSONL event.
- If the user wants to go back to Codex with a revised version, use `codex exec resume <session-id>` to continue the conversation with context preserved.

## Rules

- Always use `--full-auto` with `--json` so Codex can read the repo and output is structured
- Timeout: allow up to 180 seconds for Codex to respond (specs require more reading)
- If Codex CLI is not installed (`which codex` fails), tell the user and stop
- Do NOT paste file contents into the prompt — let Codex read the file directly from the repo
- If Codex and you disagree, present both perspectives and let the user decide
