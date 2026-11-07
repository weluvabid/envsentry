# Diagnostics

A comparison returns an ordered list of diagnostics. Each diagnostic contains a code, severity, canonical path and optional source metadata. Configuration values are never included.

## Diagnostic codes

| Code | Severity | Meaning |
|---|---|---|
| `MISSING_KEY` | Error | A path exists in the reference but not in the target. |
| `UNEXPECTED_KEY` | Warning | A path exists in the target but not in the reference. |
| `DUPLICATE_KEY` | Error | A path occurs more than once in a document. |
| `EMPTY_VALUE` | Warning | A value is empty or contains only whitespace. |
| `VALUE_MISMATCH` | Warning | Same-kind values have different stored representations. |
| `TYPE_MISMATCH` | Warning | Values at the same path have different kinds. |
| `PARSE_ERROR` | Reserved | Defined in the code enum; parser failures are CLI errors rather than comparison diagnostics. |

Duplicate diagnostics are emitted for every occurrence after the first. Missing and unexpected paths are still reported when the path is duplicated. Empty-value diagnostics are evaluated for every entry, including duplicate and unmatched entries.

## Comparison modes

`compare` checks structure, value kinds and values.

`check --schema` checks structure and empty values while ignoring value and type differences.

`--strict` changes the command's success policy: warnings cause exit code `1`. It does not change warning severity in the report.

## Source information

CLI-loaded documents include their source filename. The model supports optional one-based line and column coordinates, and the comparison engine carries supplied coordinates into diagnostics. The built-in dotenv, JSON and TOML parsers do not currently populate them.

## Ordering

Diagnostics are sorted by:

1. canonical path;
2. diagnostic code;
3. severity;
4. source name;
5. source location.

This ordering is shared by text and JSON output.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | No finding violates the selected policy. |
| `1` | Errors exist, or warnings exist with `--strict`. |
| `2` | Invalid invocation, unsupported format, file I/O failure, parsing failure or report-writing failure. |
