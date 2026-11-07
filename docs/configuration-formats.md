# Configuration formats

EnvSentry supports dotenv, JSON and TOML. Each parser converts its input into the same flattened `ConfigDocument` model.

## Format selection

The CLI selects a parser from the filename:

| Input | Parser |
|---|---|
| `.env`, `.env.*`, `.dotenv`, `.env` extension | dotenv |
| `.json` | JSON |
| `.toml` | TOML |
| Extensionless filename | dotenv |

Extensions are case-insensitive. Format detection does not inspect file contents. Other extensions are rejected.

## dotenv

Parsing uses `dotenvy` through its iterator API. Values are parsed without mutating the current process environment.

Each assignment becomes a string entry. Repeated assignments remain separate entries and are reported as duplicates. Empty and whitespace-only values are retained for diagnostics.

## JSON

JSON input must have an object at its root. Nested objects and arrays are flattened into canonical paths.

For example:

```json
{
  "database": { "port": 5432 },
  "nodes": [true, null]
}
```

produces entries for `database`, `database.port`, `nodes`, `nodes[0]` and `nodes[1]`. Container entries preserve object and array types, including empty containers.

The custom Serde visitor stores object members as a sequence rather than a map. Repeated keys therefore survive parsing, including repeated keys whose values are nested objects or arrays.

JSON strings, numbers, booleans and null retain distinct value kinds. Integer and floating-point values retain the textual representation produced by the parser.

## TOML

TOML tables and arrays use the same container-entry and indexed-path representation as JSON. TOML-native values retain their kinds, including datetimes.

The TOML parser rejects duplicate definitions as invalid TOML. Those inputs fail parsing rather than producing comparison diagnostics.

## Root model

JSON documents must be objects at the root. Root arrays and scalar values are rejected because EnvSentry models configuration as named paths. TOML documents naturally provide a root table. Dotenv documents consist of named assignments.
