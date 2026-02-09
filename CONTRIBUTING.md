# Contributing

Thanks for contributing to `grov`.

## Development Setup

1. Install stable Rust toolchain.
2. Clone the repository.
3. Run checks locally:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Branching and Pull Requests

- Do not push directly to `main`.
- Create a feature branch for every change.
- Open a pull request to `main`.
- Keep PRs focused and include tests for behavior changes.

Merges require passing CI checks and resolved review conversations.

## Commit and PR Quality

- Use clear, concise commit messages.
- Include a summary of the user-visible change in the PR description.
- Document any edge cases and migration/compatibility impact.

## Reporting Issues

Use the issue templates for bugs and feature requests.
For vulnerabilities, follow `SECURITY.md` instead of opening a public issue.
