# Security model

EnvSentry compares configuration values while keeping those values out of its reports.

## Report contents

Text and JSON reports contain diagnostic codes, severities, canonical paths, source filenames and optional source coordinates. They do not contain old values, new values or value previews.

Parser errors use generic messages and do not include parser-provided input excerpts. `ConfigValue` redacts its raw field in `Debug` output.

The CLI escapes control characters in displayed paths and filenames. JSON reports are serialized through Serde.

## Sensitivity classification

The core exposes `Sensitivity` and `SensitivityPolicy`.

The default policy recognizes common names including `PASSWORD`, `SECRET`, `TOKEN`, `API_KEY`, `DATABASE_URL` and `PRIVATE_KEY`. Matching is case-insensitive. Custom patterns support `*` as a wildcard. Public patterns take precedence over sensitive patterns when classifying a path. An explicit `Sensitive` or `Public` entry classification takes precedence over path-based classification; `Unknown` entries are classified by path.

Array indexes are omitted when constructing the dotted key used for classification, so a field such as `services[0].api_token` is classified using `SERVICES.API_TOKEN`.

The built-in parsers mark entries `Unknown`, and the CLI does not currently use classification to control output. All configuration values are omitted from reports regardless of classification.

## Data handling

The CLI reads complete input files into memory and parses them in-process. Raw values therefore exist in process memory while comparison runs. EnvSentry does not modify the process environment when parsing dotenv input.

Value-free reports prevent configuration values from being copied into normal diagnostic output. Input files, process memory, shell history and other tools that handle the files remain outside this report-handling boundary.
