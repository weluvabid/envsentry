# Design decisions

This document records the decisions that define EnvSentry's behaviour and public contract.

## One canonical model

Each parser converts format-specific input into `ConfigDocument`, `ConfigEntry`, `ConfigPath` and `ConfigValue`. The comparison engine operates only on this model. Parser changes do not require format-specific comparison logic.

## Preserve duplicate entries

Entries are stored in a vector rather than a map so repeated assignments and object members remain observable. The comparison engine groups entries by canonical path and emits a `DUPLICATE_KEY` diagnostic for each occurrence after the first.

JSON object and array containers are represented explicitly at their paths. This is necessary to detect repeated parent keys even when their values contain nested structures. TOML rejects duplicate definitions during parsing.

## Represent container structure

Objects and arrays produce entries at their own paths in addition to their descendants. This preserves empty containers and distinguishes an object or array from a scalar at the same path. Array elements use numeric path segments.

JSON roots are restricted to objects. This aligns JSON with the named-key configuration model used by dotenv and TOML.

## Compare typed values

When value comparison is enabled, the engine first compares `ValueKind`. Values of the same kind are compared using their stored string representation. EnvSentry does not perform semantic normalization such as treating differently formatted numeric strings as equivalent.

## Keep values out of diagnostics

`ConfigValue` retains raw data for comparison, but its `Debug` implementation redacts the raw field. Diagnostics contain codes, severity, paths, source names and optional locations; they never contain compared values. Text and JSON CLI renderers follow the same rule.

## Deterministic results

The comparison engine indexes paths with `BTreeMap` and sorts diagnostics by path, code, severity, source name and location. This provides stable text output, JSON output and test results.

## Separate parsing, comparison and presentation

The core has no filesystem or terminal dependencies. Parsers own syntax handling. The CLI owns file loading, format detection, report serialization and process exit codes.

## Policy and exit codes

Missing and duplicate keys are errors. Unexpected keys, empty values and value/type mismatches are warnings. Warnings do not fail a normal invocation; `--strict` makes them fail without changing their reported severity.

The `check` command disables value comparison but retains structural diagnostics and empty-value checks.

## Sensitivity classification

`SensitivityPolicy` is a core-library classification facility. Public patterns are checked before sensitive patterns; explicit entry classifications are used when an entry is classified. The CLI currently omits all values regardless of classification, so classification does not change report contents.

## Source locations

The model supports one-based line and column coordinates. Built-in parsers currently leave these coordinates unset, so CLI diagnostics identify the source file but do not include line and column.
