---
name: codex-review
description: Send current changes to Codex CLI for an independent code review
---

Get an independent code review from Codex CLI on the current changes in the repo. Follow this process:

1. **Gather the diff**: Run `git diff` (unstaged) and `git diff --cached` (staged). If both are empty, check `git diff HEAD~1` for the most recent commit. If there's still nothing, tell the user there are no changes to review and stop.

2. **Send to Codex**: Run Codex in non-interactive mode to review the changes:

   ```
   codex exec --json --full-auto "You are reviewing code changes in a Rust CLI project called grov (a git worktree manager). Review the current uncommitted or recently committed changes. Focus on: bugs, logic errors, edge cases, missing error handling, naming issues, and whether the changes follow the existing patterns in the codebase. Be concise and actionable. Skip praise — only report issues. If everything looks good, just say 'No issues found.'" 2>&1
   ```

3. **Parse the response**: Extract all `agent_message` items from the JSONL output. Ignore `reasoning`, `command_execution`, and metadata lines.

4. **Present the review**: Show Codex's feedback to the user clearly, prefixed with "**Codex Review:**". Include all substantive feedback.

5. **Act on feedback** (if requested): If the user asks you to address any of Codex's findings, fix them. Do NOT automatically apply fixes — wait for the user to decide which feedback to act on.

## Rules

- Always use `--full-auto` with `--json` so Codex can read files but output is structured
- Use `read-only` sandbox if `--full-auto` is not available
- Timeout: allow up to 120 seconds for Codex to respond
- If Codex CLI is not installed (`which codex` fails), tell the user and stop
- Do NOT send the diff as a prompt argument — let Codex read the repo directly via `git diff`
- If Codex raises a point you disagree with, say so — give the user both perspectives
