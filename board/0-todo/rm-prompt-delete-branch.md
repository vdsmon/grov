# Prompt to delete branch on grov rm

When running `grov rm`, prompt the user to also delete the branch. Attempt the deletion — if it fails (e.g., unmerged changes), inform the user and print the force-delete command (`git branch -D <branch>`) for them to run manually.
