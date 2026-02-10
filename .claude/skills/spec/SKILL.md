---
name: spec
description: Create a structured spec for a feature or change
---

Create a spec for the requested feature or change. Follow this process:

1. **Research the codebase first**: Read relevant source files to understand the current architecture, patterns, and conventions. Use the CLAUDE.md and existing code as your guide. Do this silently before engaging the user.

2. **Ask questions — NEVER assume**: Before writing anything, identify every ambiguity, design decision, and trade-off in the request. Ask the user about ALL of them using the AskUserQuestion tool. This is mandatory, not optional.

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

3. **Write the spec**: Only after all questions are answered, create a new file in `specs/` following the template at `specs/TEMPLATE.md`. Name it with a sequential number and kebab-case title (e.g., `specs/001-feature-name.md`).

4. **Fill in all sections**:
   - Write a clear user story
   - Define concrete, testable acceptance criteria
   - Identify all affected files with brief rationale
   - Describe the technical approach, referencing existing patterns
   - List edge cases
   - Break work into 3-7 small, independent tasks — each implementable in one session
   - Specify what tests are needed
   - Note what is explicitly out of scope

5. **Set status to `draft`** and present the spec for review.

Guidelines:
- Each task should be self-contained: an agent reading only the spec and the codebase should be able to implement it
- Reference specific files and functions, not vague areas
- Keep tasks small — prefer "add function X to file Y" over "implement feature Z"
- Include the exact command-line interface changes (flags, subcommands) if applicable
- Note any breaking changes or migration needs
- Every design decision in the spec must trace back to an explicit user answer, not an assumption
