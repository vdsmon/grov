#!/usr/bin/env zsh

# Local grov dev environment
_grov_dev_script_path="${(%):-%x}"
_grov_dev_script_dir="${_grov_dev_script_path:A:h}"

export GROV_DEV_ROOT="${GROV_DEV_ROOT:-$HOME/.local/grov-dev}"
if [[ -n "${TMPDIR:-}" ]]; then
  export GROV_E2E_ROOT="${GROV_E2E_ROOT:-${TMPDIR%/}/grov-e2e}"
else
  export GROV_E2E_ROOT="${GROV_E2E_ROOT:-}"
fi
export GROV_REPO="${GROV_REPO:-${_grov_dev_script_dir:h}}"

unset _grov_dev_script_path _grov_dev_script_dir

grovd() {
  local subcmd="${1:-}"

  case "$subcmd" in
    "")
      echo "Usage: grovd <command> [args...]"
      echo ""
      echo "Meta-commands:"
      echo "  refresh           Build and install dev binary"
      echo "  sandbox [--reset] cd into sandbox (optionally wipe first)"
      echo "  repo              cd to source repo"
      echo ""
      echo "Everything else is forwarded to the dev binary."
      ;;

    refresh)
      local build_from="$GROV_REPO"
      if [[ -f "$PWD/Cargo.toml" ]] && grep -q '^name = "grov"' "$PWD/Cargo.toml" 2>/dev/null; then
        build_from="$PWD"
      fi
      echo "Building from $build_from ..."
      cargo install --path "$build_from" --root "$GROV_DEV_ROOT" --force
      ;;

    sandbox)
      shift
      if [[ "${1:-}" == "--reset" ]]; then
        echo -n "Wipe $GROV_E2E_ROOT? [y/N] "
        read -r reply
        if [[ "$reply" != [yY] ]]; then
          echo "Aborted."
          return 0
        fi

        local tmp_root="${TMPDIR:-}"

        if [[ -z "$tmp_root" ]]; then
          echo "Refusing: TMPDIR is not set."
          return 1
        fi

        tmp_root="${tmp_root%/}"

        if [[ -z "$GROV_E2E_ROOT" ]]; then
          echo "Refusing: GROV_E2E_ROOT is empty."
          return 1
        fi

        if [[ "$GROV_E2E_ROOT" != "$tmp_root"/* ]]; then
          echo "Refusing: GROV_E2E_ROOT must be under TMPDIR root ($tmp_root)."
          return 1
        fi

        if [[ "${GROV_E2E_ROOT:t}" != "grov-e2e" ]]; then
          echo "Refusing: reset is restricted to paths ending in /grov-e2e."
          return 1
        fi

        rm -rf -- "$GROV_E2E_ROOT"
        mkdir -p -- "$GROV_E2E_ROOT"
        echo "Sandbox reset at $GROV_E2E_ROOT"
      fi

      if [[ -z "$GROV_E2E_ROOT" ]]; then
        echo "GROV_E2E_ROOT is not set (TMPDIR may be unset)."
        return 1
      fi

      mkdir -p -- "$GROV_E2E_ROOT"
      cd -- "$GROV_E2E_ROOT" || return 1
      echo "Now in $GROV_E2E_ROOT"
      ;;

    repo)
      cd -- "$GROV_REPO" || return 1
      echo "Now in $GROV_REPO"
      ;;

    *)
      local grov_bin="$GROV_DEV_ROOT/bin/grov"
      if [[ ! -x "$grov_bin" ]]; then
        echo "grov dev binary not found at $grov_bin"
        echo "Run: grovd refresh"
        return 1
      fi

      "$grov_bin" "$@"
      ;;
  esac
}
