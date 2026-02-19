# Interactive prompts for missing required arguments

When a required positional argument is omitted in a TTY session, prompt the user interactively instead of printing a usage error. This applies to:

- **`grov add`** — prompt for the branch name (could offer a list of remote branches)
- **`grov remove`** — prompt for the worktree to remove (could offer a list of existing worktrees)

`grov init` already handles interactive prompts for missing flags. `grov completions` is a developer utility and doesn't need this treatment.

In non-TTY contexts, these commands should still fail with a clear error and usage hint, since there's no one to prompt.
