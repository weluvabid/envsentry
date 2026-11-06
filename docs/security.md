# Security and secret handling

EnvSentry is built around a simple rule: finding configuration drift shouldn't mean copying configuration values into logs.

## What reports contain

Text and JSON reports include diagnostic metadata:

- diagnostic code and severity;
- canonical configuration path;
- source filename, when known;
- line and column, when supplied by the caller.

They do **not** include the old value, the new value, or a preview of either. Parse errors are generic too, so malformed input doesn't get echoed back with a secret attached.

`ConfigValue` stores the raw value because comparisons need it. Its `Debug` implementation prints `[REDACTED]` instead of the contents. This is useful protection against accidental debug logging, but it isn't a substitute for being careful with values elsewhere in an application.

## Sensitivity metadata

The core exposes a `Sensitivity` enum and `SensitivityPolicy`. The default policy recognises common names such as `PASSWORD`, `SECRET`, `TOKEN`, `API_KEY`, `DATABASE_URL`, and `PRIVATE_KEY`. Callers can add sensitive or public wildcard patterns, and explicit entry classifications are respected.

This is a classification API. The built-in parsers mark entries as `Unknown`, and the CLI does not apply the policy when producing reports. Classification therefore does not change CLI output: reports omit configuration values for every entry.

If a real use case emerges for displaying non-sensitive values, configurable redaction could be added as a separate reporting option. The current behaviour remains simple: reports never include configuration values.

## Practical boundaries

- Paths and source filenames can originate in input or command-line arguments. The CLI escapes control characters in text output to avoid terminal-control surprises.
- JSON output is serialized as JSON rather than assembled by string concatenation.
- File contents are read into memory for parsing and comparison. Avoid pointing EnvSentry at untrusted, enormous files in environments where memory use needs a hard limit.
- Value-free diagnostics reduce accidental disclosure, but they don't protect the input files themselves, process memory, shell history, or other tools that may handle those files.

In short: EnvSentry keeps values out of its reports. It doesn't claim to be a secret manager or a sandbox.
