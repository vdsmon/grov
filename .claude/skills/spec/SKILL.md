---
name: spec
description: Create a structured spec for a feature or change
---

Create a spec for the requested feature or change. Follow this process:

1. **Pick or receive the idea**:
   - If the user provided arguments (a description or a `board/todo/` file path), use that directly — skip the picker.
   - If no arguments were provided, scan `board/todo/` for `.md` files.
     - If there are todos: present a picker using AskUserQuestion. Show up to 5 files (most recent first by filename). Each option label is the todo filename (without `.md`), and the description is the `# Title` from the file. The user can also type a fresh idea via the "Other" free-text option.
     - If there are no todos: skip the picker and ask the user to describe the idea.
   - If the selected option is a `board/todo/` file, read it as the idea input and remember the path — it will be deleted after the spec is written.

2. **Research the codebase**: Read relevant source files to understand the current architecture, patterns, and conventions. Use the CLAUDE.md and existing code as your guide. Do this silently before engaging the user.

3. **Ask questions — NEVER assume**: Before writing anything, identify every ambiguity, design decision, and trade-off in the request. Ask the user about ALL of them using the AskUserQuestion tool. This is mandatory, not optional.

   You MUST ask about:
   - Behavioral details: What exactly should happen? What should the output look like?
   - Edge cases: What happens when things go wrong or inputs are unexpected?
   - Scope boundaries: What is included vs excluded?
   - Technical constraints: Are there platform limitations, compatibility concerns, or architectural preferences?
   - UX decisions: Exact wording of prompts/messages, flag names, command syntax
   - Breaking changes: Is it acceptable to change existing behavior?

   Do NOT:
   - Assume a default when the user hasn't specified one
   - Infer intent from vague descriptions
   - Fill in blanks with "reasonable" guesses
   - Write the spec and then ask "does this look right?" — ask FIRST, write AFTER

4. **Write the spec**: Only after all questions are answered, create a new file in `board/specs/active/` following the template at `board/TEMPLATE.md`. Name it with a sequential number and kebab-case title (e.g., `board/specs/active/003-feature-name.md`). Use the next available number across both `board/specs/active/` and `board/specs/done/`.

5. **Fill in all sections**:
   - Write a clear user story
   - Define concrete, testable acceptance criteria
   - Identify all affected files with brief rationale
   - Describe the technical approach, referencing existing patterns
   - List edge cases
   - Break work into 3-7 small, independent tasks — each implementable in one session
   - Specify what tests are needed
   - Note what is explicitly out of scope

6. **Set status to `draft`** and present the spec for review.

7. **Delete the source todo**: If a `board/todo/` file was consumed in step 1, delete it now. Git history preserves the original idea.

Guidelines:
- Each task should be self-contained: an agent reading only the spec and the codebase should be able to implement it
- Reference specific files and functions, not vague areas
- Keep tasks small — prefer "add function X to file Y" over "implement feature Z"
- Include the exact command-line interface changes (flags, subcommands) if applicable
- Note any breaking changes or migration needs
- Every design decision in the spec must trace back to an explicit user answer, not an assumption
