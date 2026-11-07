# Architecture

EnvSentry is a Rust workspace with three crates and a one-way dependency flow.

```text
envsentry-cli
  ├── envsentry-parsers
  │     └── envsentry-core
  └── envsentry-core
```

## Crate responsibilities

- **`envsentry-core`** defines configuration paths, typed values, documents, comparison options, diagnostics and sensitivity classification. It does not perform file I/O or render output.
- **`envsentry-parsers`** parses dotenv, JSON and TOML into the shared core model. Format-specific syntax and duplicate handling remain in this crate.
- **`envsentry-cli`** parses arguments, reads files, selects parsers, renders text or JSON and maps findings to process exit codes.

## Data flow

1. The CLI reads each input as UTF-8 text.
2. The input filename selects a parser.
3. The parser creates a `ConfigDocument` containing ordered `ConfigEntry` values.
4. The core indexes entries by `ConfigPath` and compares the reference against the target.
5. The CLI renders diagnostics without configuration values and selects an exit code.

## Configuration model

A `ConfigDocument` contains a format, an optional source name and a vector of entries. Each entry contains:

- a canonical `ConfigPath`;
- a `ConfigValue` and its `ValueKind`;
- sensitivity metadata;
- an optional source location;
- an occurrence number.

Objects and arrays are represented by entries at their own paths as well as entries for their descendants. This preserves container type and makes repeated container keys observable. Array elements use indexed path segments.

Path rendering escapes dots, brackets, backslashes and control characters in literal keys. This keeps literal keys distinct from structural path separators and array indexes.

## Comparison model

The core builds `BTreeMap` indexes keyed by `ConfigPath`. It reports missing paths, unexpected paths and repeated entries. For paths occurring exactly once in both documents, value comparison checks the value kind first and then the stored textual representation. Empty and whitespace-only values are evaluated independently.

Diagnostics are sorted by path, code, severity, source name and location. The same documents and options therefore produce the same diagnostic order.

## Boundaries

The current CLI is synchronous and loads complete files into memory. Built-in parsers do not attach line and column locations. Sensitivity classification is available in the core API but does not alter CLI reports; all reports omit configuration values.
