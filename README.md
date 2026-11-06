# EnvSentry

A small Rust CLI for spotting configuration drift without dumping your secrets into the terminal. Because CI logs have enough problems already.

EnvSentry compares dotenv, JSON, and TOML files through one common model. It can check both values and structure, or just check that a target has the keys your schema expects.

## Quick start

Compare a checked-in example with the config you're actually using:

```sh
envsentry compare .env.example .env
```

Check a target against a separate schema file, ignoring value differences:

```sh
envsentry check --schema config.schema.toml config.toml
```

For CI, emit JSON:

```sh
envsentry compare reference.json target.json --format json
```

Warnings are allowed by default. Make them fail the command with `--strict`:

```sh
envsentry compare .env.example .env --strict --no-color
```

## Supported formats

- **dotenv:** `.env`, `.env.*`, `.dotenv`, `.env` extension, and extensionless files
- **JSON:** `.json`
- **TOML:** `.toml`

Nested objects, tables, and arrays are normalized into canonical paths. Duplicate dotenv assignments and JSON object keys are preserved so they can be diagnosed; TOML duplicate definitions are rejected by the TOML parser.

## What gets reported

EnvSentry reports diagnostic codes, severities, paths, and source filenames. Line and column fields are supported by the model, but the built-in parsers don't populate them.

Configuration values are never included in text or JSON reports. Parse errors are generic and value-free too. Control characters in displayed paths and filenames are escaped before text output.

Missing and duplicate keys are errors. Unexpected keys, empty values, and value/type mismatches are warnings. `--strict` makes warnings fail the command. The `check` command ignores value and type differences, but still reports structural findings and empty values.

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | No policy failure |
| `1` | Findings violate the selected policy |
| `2` | Invalid invocation, unsupported format, file I/O failure, or parsing failure |


## Documentation

- [Architecture](docs/architecture.md) — the crates, data flow, and common model
- [Design decisions](docs/design-decisions.md) — why the project is shaped this way
- [Configuration formats](docs/configuration-formats.md) — parser behaviour and normalization
- [Diagnostics](docs/diagnostics.md) — codes, severity, ordering, and source locations
- [Security](docs/security.md) — what is and isn't exposed
- [Development](docs/development.md) — toolchain, checks, and tests

## Development

Use the Rust toolchain pinned in `rust-toolchain.toml`. Run the same formatting, lint, and test checks used by CI:

```sh
./scripts/check.sh
```

Build the workspace or inspect the CLI help:

```sh
cargo build --workspace --locked
cargo run --locked -p envsentry-cli -- --help
```
