# How EnvSentry is put together

EnvSentry is split into three Rust crates. That's not architecture astronaut stuff; each one has a job, and the boundaries keep the comparison engine from caring about file syntax or terminal output.

## The three pieces

- **`envsentry-core`** owns the common configuration model, comparison rules, diagnostics, and sensitivity metadata. It doesn't read files or print reports.
- **`envsentry-parsers`** turns dotenv, JSON, and TOML input into the core model. Each parser deals with its format's quirks, then hands over the same shape of data.
- **`envsentry-cli`** handles arguments, file loading, format detection, output, and process exit codes.

The flow is:

```text
input files
    |
    v
CLI: read files + identify format
    |
    v
Parsers: format-specific input -> ConfigDocument
    |
    v
Core: compare canonical paths -> diagnostics
    |
    v
CLI: text or JSON report -> exit code
```

## The common model

A `ConfigDocument` contains a format, an optional source name, and a list of `ConfigEntry` values. Each entry has:

- a canonical `ConfigPath`;
- a `ConfigValue` and its `ValueKind`;
- sensitivity metadata;
- an optional source location;
- an occurrence number, so repeated dotenv assignments don't vanish.

Nested objects and arrays are flattened into paths such as `database.port` and `nodes[0]`. Keys containing dots, brackets, backslashes, or control characters are escaped when rendered. That keeps a literal key like `items[0]` distinct from the first element of an `items` array.

Values stay in the model because comparisons need them. Their `Debug` representation is redacted, and diagnostics deliberately carry metadata rather than the values themselves.

## Comparison

The core builds ordered indexes keyed by `ConfigPath`, then checks for missing, unexpected, and duplicate entries. When value comparison is enabled, it also checks value kinds and contents. Empty values are checked independently, including on duplicated or otherwise unmatched entries.

The ordered map and final diagnostic sort make output stable. Same inputs, same findings, same order. CI logs appreciate boring consistency.

## A note on metadata

The model supports sensitivity classifications and source locations. The CLI does not use sensitivity classifications to change reporting, and the built-in parsers do not populate line and column locations. Those boundaries are described in [Security](security.md) and [Diagnostics](diagnostics.md); they are model capabilities, not automatic behaviour.
