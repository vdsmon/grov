#!/usr/bin/env zsh
set -euo pipefail

script_dir="${0:A:h}"
env_script="$script_dir/grov-dev-env.sh"
zshrc="${ZDOTDIR:-$HOME}/.zshrc"

env_script_ref="$env_script"
if [[ "$env_script" == "$HOME/"* ]]; then
  env_script_ref="\$HOME/${env_script#$HOME/}"
fi

source_line="source \"$env_script_ref\""

if [[ ! -f "$env_script" ]]; then
  echo "Missing helper: $env_script"
  exit 1
fi

touch "$zshrc"

if grep -Fqx "$source_line" "$zshrc"; then
  echo "Already configured in $zshrc"
else
  tmp_file="$(mktemp)"
  awk '!/grov-dev-env\.sh/' "$zshrc" > "$tmp_file"
  printf '\n%s\n' "$source_line" >> "$tmp_file"
  mv "$tmp_file" "$zshrc"
  echo "Updated $zshrc with:"
  echo "  $source_line"
fi

echo "Run: source \"$zshrc\""
