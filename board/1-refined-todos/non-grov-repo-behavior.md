# Graceful behavior on non-grov-initialized repos

Define and implement clear behavior when grov commands run outside a grov-managed project. Three scenarios to handle:

1. **No bare repo found** — clear error message explaining this isn't a grov project, with a hint to run `grov init`.
2. **Bare repo exists but isn't named `repo.git`** — same as above; grov only recognizes `repo.git`.
3. **`repo.git` exists but has no `.grov.toml`** — this could be a manually created bare repo. Either offer to adopt it (write a `.grov.toml`) or error with a message explaining what's missing.

The goal is helpful error messages that guide the user toward the right action, not silent failures or cryptic git errors.
