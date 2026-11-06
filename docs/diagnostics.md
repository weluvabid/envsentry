# Diagnostics

Diagnostics are the public result of a comparison. They tell you what needs attention without printing the values that caused the finding.

## Current diagnostic codes

| Code | Severity | Meaning |
| --- | --- | --- |
| `MISSING_KEY` | Error | A path exists in the reference but not in the target. |
| `UNEXPECTED_KEY` | Warning | A path exists in the target but not in the reference. |
| `DUPLICATE_KEY` | Error | A path occurs more than once in a document. |
| `EMPTY_VALUE` | Warning | A value is empty or contains only whitespace. |
| `VALUE_MISMATCH` | Warning | Values differ while their kinds match. |
| `TYPE_MISMATCH` | Warning | The values have different kinds. |
| `PARSE_ERROR` | Reserved | Present in the code enum; parser failures surface as CLI errors and exit with code `2`, rather than comparison report entries. |

`--strict` promotes warnings to command failure. It doesn't rewrite their severity in the report.

## Source information

Every parsed CLI document gets a source filename. The model also supports an optional `SourceLocation` (one-based line and column), and the comparison engine carries that location into diagnostics when the relevant entry has one.

One detail worth knowing: the built-in dotenv, JSON, and TOML parsers don't populate line/column coordinates. CLI reports include the filename, but not line and column for parser-produced entries. Callers constructing `ConfigEntry` values themselves can attach a location with `with_source`.

When a diagnostic is created, the location comes from the entry most directly associated with the finding: the reference entry for a missing key, the target entry for an unexpected key or duplicate, and the target entry for value/type differences. Empty-value diagnostics use the entry that is empty.

## Stable ordering

Diagnostics are sorted by path, code, severity, source name, and location before being returned. That makes output predictable across runs and keeps CI diffs from turning into confetti.
