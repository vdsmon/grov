# Local Dev Setup (`grovd`)

This guide explains how to test the in-development `grov` binary on your machine without touching the published crates.io install.

## Goal

- keep your normal `grov` install intact
- run a local dev binary via `grovd`
- keep manual test projects isolated under `$GROV_E2E_ROOT`

## One-time setup

From the repo root, run:

```sh
./scripts/grov-dev-setup-zsh.sh
source ~/.zshrc
```

The setup script auto-detects the current repo path and updates `~/.zshrc` with the correct `source` line.

Manual alternative (if you prefer writing the line yourself from current directory):

```sh
grep -Fqx "source \"$PWD/scripts/grov-dev-env.sh\"" ~/.zshrc || \
  echo "source \"$PWD/scripts/grov-dev-env.sh\"" >> ~/.zshrc
source ~/.zshrc
```

## What the helper provides

The script defines these defaults:

- `GROV_DEV_ROOT="${GROV_DEV_ROOT:-$HOME/.local/grov-dev}"`
- `GROV_E2E_ROOT="${GROV_E2E_ROOT:-${TMPDIR%/}/grov-e2e}"` (when `TMPDIR` is set)
- `GROV_REPO="${GROV_REPO:-<repo-root-derived-from-script>}"`

And a single dispatcher function:

- `grovd refresh` — build and install the dev binary into `$GROV_DEV_ROOT`
- `grovd sandbox` — cd into `$GROV_E2E_ROOT` (creates it if needed)
- `grovd sandbox --reset` — wipe and recreate `$GROV_E2E_ROOT`, then cd into it
- `grovd repo` — cd back to the source repo
- `grovd <anything else>` — passthrough to the dev binary at `$GROV_DEV_ROOT/bin/grov`

## Daily workflow

1. Rebuild dev binary when code changes:

   ```sh
   grovd refresh
   ```

2. Reset the sandbox at the start of a manual test session:

   ```sh
   grovd sandbox --reset
   ```

3. Start an isolated project test:

   ```sh
   grovd init --url <repo-url> --prefix <prefix>
   ```

   Omit flags to test interactive prompts:

   ```sh
   grovd init
   ```

4. Continue testing inside the created worktree:

   ```sh
   cd <project>/<prefix>_main
   grovd add feature/sandbox
   grovd list
   grovd remove feature/sandbox --delete-branch
   ```

5. Return to the source repo:

   ```sh
   grovd repo
   ```

## Validation checks

Confirm binary isolation:

```sh
which grov
type grovd
grovd --version
```

Expected behavior:

- `which grov` points to your normal install (for example `~/.cargo/bin/grov`)
- `grovd` resolves to a shell function from `scripts/grov-dev-env.sh`
- `grovd --version` runs the binary from `~/.local/grov-dev/bin/grov`

Confirm sandbox isolation:

```sh
grovd sandbox --reset
find "$GROV_E2E_ROOT" -maxdepth 2 -type d | sort
```

All test artifacts should stay under `$GROV_E2E_ROOT` unless you pass explicit custom paths.

## Safety behavior

`grovd sandbox --reset` refuses to run if:

- `GROV_E2E_ROOT` is empty
- `TMPDIR` is not set
- `GROV_E2E_ROOT` is outside `TMPDIR`
- `GROV_E2E_ROOT` does not end with `/grov-e2e`

This prevents accidental deletion of unrelated directories.

## Troubleshooting

- `grovd: command not found`
  - run `source ~/.zshrc`
- `grov dev binary not found at ...`
  - run `grovd refresh`
- `directory already exists` during `grovd init`
  - run `grovd sandbox --reset` or choose a different `--name`
