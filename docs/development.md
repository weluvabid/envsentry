# Development

A few commands to get moving without having to reverse-engineer the workflow first.

## Requirements

Use the Rust toolchain pinned to `1.98.1` in `rust-toolchain.toml`, with the `rustfmt` and `clippy` components.

## Git hooks

Install the repository's pre-commit hook once after cloning:

```sh
./scripts/install-hooks.sh
```

The hook runs `./scripts/check.sh` before each commit. That includes formatting, Clippy, and the full workspace test suite; if any check fails, the commit is stopped. Cargo must be available locally.

## Checks

Run the same checks as CI manually:

```sh
./scripts/check.sh
```

That script runs:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
```

The lockfile is used in CI and in the documented build commands. If dependencies change, update `Cargo.lock` and commit it with the change.

## Workspace layout

```text
.githooks/
  pre-commit           # mandatory local quality gate before commits
crates/
  envsentry-core/      # model, comparison, diagnostics, sensitivity metadata
  envsentry-parsers/   # dotenv, JSON, TOML
  envsentry-cli/       # arguments, file loading, renderers, exit codes
scripts/
  check.sh             # local/CI quality gate
  install-hooks.sh     # configure the repository Git hooks
```

## Tests worth keeping an eye on

The unit tests cover normalization, duplicate handling, empty structures and values, deterministic diagnostics, and safe error/debug output. CLI integration tests execute the built binary and check exit codes, JSON output, strict mode, schema checks, and that known secret sentinels don't leak into output.

When adding a feature, test the behaviour at the boundary where users observe it. For security-related changes, include a sentinel value and assert it does not appear in stdout, stderr, or debug output.
