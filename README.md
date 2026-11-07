# EnvSentry

EnvSentry is a Rust CLI for detecting configuration drift across dotenv, JSON and TOML files. It reports structural and value differences without including configuration values in its output.

![EnvSentry CLI demonstration](demo/demo.gif)

## Usage

Compare a target configuration against a reference:

```sh
envsentry compare .env.example .env
```

Check a configuration against a schema while ignoring value and type differences:

```sh
envsentry check --schema config.schema.toml config.toml
```

Emit machine-readable diagnostics for CI:

```sh
envsentry compare reference.json target.json --format json
```

Treat warnings as policy failures:

```sh
envsentry compare .env.example .env --strict --no-color
```

## Supported formats

| Format | Detection | Behaviour |
|---|---|---|
| dotenv | `.env`, `.env.*`, `.dotenv`, `.env` extension and extensionless filenames | Parses assignments without modifying the process environment; preserves repeated assignments. |
| JSON | `.json` | Requires an object root; preserves repeated object keys, including nested container keys. |
| TOML | `.toml` | Preserves native value kinds; duplicate definitions are rejected by the TOML parser. |

Nested objects, tables and arrays are normalized into canonical paths. Container entries are retained alongside their descendants, preserving empty structures and repeated parent keys.

## Diagnostics and exit codes

| Finding | Severity |
|---|---|
| Missing key | Error |
| Duplicate key | Error |
| Unexpected key | Warning |
| Empty or whitespace-only value | Warning |
| Value mismatch | Warning |
| Type mismatch | Warning |

`--strict` makes warnings fail the command without changing their reported severity. The `check` command ignores value and type differences while retaining structural and empty-value findings.

| Exit code | Meaning |
|---|---|
| `0` | No finding violates the selected policy |
| `1` | Errors exist, or warnings exist with `--strict` |
| `2` | Invalid invocation, unsupported format, file I/O, parsing or report-writing failure |

Reports contain diagnostic metadata and never include configuration values. See [Security](docs/security.md) for the report-handling boundary and sensitivity classification.

## Documentation

- [Architecture](docs/architecture.md) — crate responsibilities, data flow and configuration model
- [Design decisions](docs/design-decisions.md) — comparison semantics and implementation choices
- [Configuration formats](docs/configuration-formats.md) — parser behaviour and normalization
- [Diagnostics](docs/diagnostics.md) — diagnostic codes, severity, ordering and exit codes
- [Security](docs/security.md) — secret handling and sensitivity classification
- [Development](docs/development.md) — toolchain, quality checks and tests

## Development

Use the pinned toolchain and run the CI checks locally:

```sh
./scripts/check.sh
```

Build the workspace or inspect CLI help:

```sh
cargo build --workspace --locked
cargo run --locked -p envsentry-cli -- --help
```
