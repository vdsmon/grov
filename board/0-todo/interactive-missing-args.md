# Interactive prompts for missing arguments

Commands like `grov add` and `grov remove` should not fail when the required argument is omitted. Instead, interactively prompt the user for the missing value (e.g., branch name). All commands should gracefully fall back to interactive mode when arguments are missing.
