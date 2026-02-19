# Copy untracked files into new worktrees

When `grov add` (or `grov init`) creates a worktree, files like `.env` and other local config don't carry over since they're gitignored. Provide a mechanism to automatically copy specified untracked files from an existing worktree into the new one.

Key UX questions for `/spec` to resolve:
- Where is the file list configured? (`.grov.toml` seems natural, e.g. `[worktree] copy_files = [".env", ".env.local"]`)
- Which source worktree to copy from? (most recently used? explicit flag? first available?)
- What happens on conflicts or missing source files?
- Should there be a confirmation prompt, or just copy silently with a summary?
