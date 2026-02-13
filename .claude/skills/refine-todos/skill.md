---
name: refine-todos
description: Review, merge, and refine todos before speccing
---

Scan the todo queue, suggest groupings, and produce refined todos. Follow this process:

1. **Scan all todos**: Read every `.md` file in `board/0-todo/`. For each file, extract the `# Title` and the description body.

2. **Analyze and suggest groupings**: Look for related, overlapping, or complementary ideas. Present your analysis to the user using AskUserQuestion:
   - Show suggested **merge groups** (2+ todos that belong together) with a brief rationale for each group
   - Show **solo refinement candidates** (todos that would benefit from sharpening even without merging)
   - Show **already good** todos that don't need changes
   - Let the user pick which action(s) to take. Use multiSelect so they can approve multiple groups/refinements at once.

3. **For each approved action, refine actively**:
   - **Merging**: Combine the ideas into a single, cohesive todo. Don't just concatenate — synthesize. Sharpen the scope, add useful context from the codebase, suggest clearer framing, and resolve any contradictions between the source todos.
   - **Solo refining**: Improve clarity, add context, sharpen scope, and tighten the description. The refined version should give `/spec` a much better starting point than the raw capture.
   - Keep the todo format simple: `# Title` followed by a brief description. Stay under 300 words.
   - Generate a new kebab-case slug that reflects the merged/refined idea.

4. **Write results**: For each action:
   - Write the new/updated file to `board/1-refined-todos/new-slug.md`
   - Delete all consumed source files from `board/0-todo/`
   - Git history preserves the originals — deletion is safe

5. **Report**: Show the user what was created and what was deleted. Include a brief summary of each refined todo.

## Rules

- Do NOT research the codebase deeply — light context is fine for improving framing, but save deep research for `/spec`
- Do NOT ask design questions or make architectural decisions — that's `/spec`'s job
- Do NOT create specs, task lists, or do any implementation planning
- The output is always `board/1-refined-todos/` files, never specs
- When merging, the new todo should feel like one cohesive idea, not a list of stapled-together ideas
- If the user disagrees with a suggested grouping, respect that — don't push back
- If there are 0-1 todos and none need refinement, say so and exit
