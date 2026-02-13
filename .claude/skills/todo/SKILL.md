---
name: todo
description: Capture a raw idea to the board todo queue
---

Capture the user's raw idea into `board/todo/`. Follow this process:

1. **Generate a slug**: Create a short kebab-case slug from the idea (e.g., `add-base-branch-prompt`). Keep it under 5 words.

2. **Polish lightly**: Fix grammar and spelling in the user's input. Do NOT expand, analyze, research the codebase, or ask clarifying questions. Keep the idea's original intent and scope intact.

3. **Write the file**: Create `board/todo/slug.md` with this format:

   ```
   # Title

   Brief description of the idea.
   ```

   That's it — no template, no frontmatter, no sections. Keep the total content under 200 words.

4. **If the user provides multiple ideas**: Create one file per idea, each with its own slug.

5. **Report**: Show the user the file path(s) created.

## Rules

- Do NOT research the codebase
- Do NOT ask clarifying questions
- Do NOT create specs or do any design work
- Do NOT modify any existing files
- Keep it brief — this is rapid idea capture, not refinement
