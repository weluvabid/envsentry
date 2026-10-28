# EnvSentry

A lightweight Rust CLI for detecting configuration drift without exposing secrets.

## Supported formats

- dotenv (`.env`, `.env.*`, `.dotenv`, and extensionless files)
- JSON (`.json`)
- TOML (`.toml`)

Reports contain diagnostic codes, severities, configuration paths, source filenames, and source locations when available. Configuration values are never included in text or JSON reports. Empty nested objects, tables, and arrays are retained during normalization so structural comparisons do not silently discard them. Control characters in displayed paths are escaped to keep terminal output on one line and prevent terminal-control injection.

## Commands

Compare values and structure against a reference:

```sh
envsentry compare .env.example .env
```

Validate a target's structure against a schema while ignoring value differences:

```sh
envsentry check --schema config.toml config.toml
```

Emit machine-readable JSON for CI:

```sh
envsentry compare reference.json target.json --format json
```

Treat warnings as failures and disable terminal colors:

```sh
envsentry compare .env.example .env --strict --no-color
```

## Exit codes

- `0`: no policy failure
- `1`: findings violate the selected policy
- `2`: invalid invocation, unsupported input format, file I/O failure, or parsing failure

Missing and duplicate keys are errors. Unexpected keys, empty values, and value/type mismatches are warnings. `--strict` makes warnings fail the command. The `check` command ignores value and type differences, but still reports structural findings and empty values.

`--redact-all` is accepted for policy compatibility; current reports never emit configuration values regardless of this option.

## Build and test

```sh
cargo build --workspace
cargo test --workspace
cargo run -p envsentry-cli -- --help
```
