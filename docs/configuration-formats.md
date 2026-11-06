# Configuration formats

All supported formats are normalized into the same flattened path model before comparison.

## dotenv

Recognised filenames include `.env`, `.env.*`, `.dotenv`, files with the `.env` extension, and extensionless files.

Parsing is delegated to `dotenvy`. EnvSentry consumes the parser iterator directly rather than loading values into the current process environment. Repeated assignments are retained as repeated entries and can be reported as duplicates.

Dotenv values are strings. An empty assignment (`NAME=`) is retained and reported as an empty value.

## JSON

Files ending in `.json` are parsed with Serde. Objects and arrays become canonical paths:

```json
{
  "database": { "port": 5432 },
  "nodes": [true, null]
}
```

becomes paths like `database.port`, `nodes[0]`, and `nodes[1]`.

Duplicate object keys are preserved by the custom deserializer so EnvSentry can report them. Empty nested objects and arrays are also retained instead of disappearing during flattening. JSON strings, numbers, booleans, and null keep distinct value kinds.

## TOML

Files ending in `.toml` are parsed with `toml`. Tables and arrays are flattened in the same way as JSON, and TOML-native value kinds (including datetimes) are retained.

The TOML parser rejects duplicate definitions as invalid TOML. Unlike dotenv and JSON, those duplicates don't reach the comparison engine as repeated entries.

## Format detection

The CLI chooses a parser from the filename extension and special dotenv filename patterns. Detection is case-insensitive for extensions. YAML and other formats are not supported.

The parsers return generic, value-free errors. The CLI adds the source filename when reporting a parse failure.
