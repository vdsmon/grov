# Print cd hint after worktree creation

After `grov init` and `grov add` create a worktree, print a copy-pasteable `cd` command pointing to the new worktree directory. This lets the user jump straight into the worktree without manually constructing the path.

Both commands already print success output — this adds a `cd <path>` line that's easy to select and paste. The path is already known at the point of success, so this is a small output change with no new logic.
