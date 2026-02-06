---
name: ci-check
description: Run the full CI validation pipeline (fmt + clippy + test)
---

Run the full CI check pipeline for the grov project:

1. Run `cargo fmt --check` to verify formatting
2. Run `cargo clippy --all-targets -- -D warnings` to check for lint issues
3. Run `cargo test` to run all unit and integration tests

If formatting fails, run `cargo fmt` first, then re-run the pipeline.
Report results clearly — pass/fail for each step.
