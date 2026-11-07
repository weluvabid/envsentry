# Development

## Toolchain

The repository pins Rust `1.98.1` in `rust-toolchain.toml`, including the `rustfmt` and `clippy` components.

## Quality checks

Run the same checks used by CI:

```sh
./scripts/check.sh
```

The script executes:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
```

Build the workspace and inspect CLI usage:

```sh
cargo build --workspace --locked
cargo run --locked -p envsentry-cli -- --help
```

## Git hooks

Install the repository pre-commit hook:

```sh
./scripts/install-hooks.sh
```

The hook runs `./scripts/check.sh` before each commit. A failed check blocks the commit.

## Workspace

```text
crates/
  envsentry-core/       # configuration model, comparison and diagnostics
  envsentry-parsers/    # dotenv, JSON and TOML parsers
  envsentry-cli/        # arguments, file loading, renderers and exit codes
scripts/
  check.sh              # formatting, lint and test checks
  install-hooks.sh      # configures the repository Git hooks
.githooks/
  pre-commit            # runs the quality checks
```

## Test strategy

Core tests cover model invariants, comparison semantics, diagnostic ordering and sensitivity classification. Parser tests cover normalization, duplicate preservation, empty containers, value kinds and value-free parse errors. CLI integration tests exercise process exit codes, strict mode, schema checks, JSON output and secret-sentinel handling.

When changing parsing or reporting, test the observable behaviour at the parser or CLI boundary. Security-related tests should use sentinel values and assert that they are absent from stdout, stderr and debug output.
