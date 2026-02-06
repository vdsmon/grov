---
name: release
description: Prepare and execute a grov release
disable-model-invocation: true
---

Guide the user through a grov release:

1. Verify working tree is clean (`git status`)
2. Run full CI check (`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`)
3. Ask user for bump level: patch, minor, or major
4. Run `cargo release <level> --no-publish --no-confirm --execute`
5. Confirm tag was created and pushed

Important: cargo-release requires a clean working tree. If there are untracked files, warn the user to gitignore or commit them first.
