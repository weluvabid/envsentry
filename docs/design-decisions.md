# Design decisions (and a few reasons)

This is the short version of why EnvSentry is shaped the way it is. Not every choice needs a 20-page ADR; some just need a clear explanation and a place to point curious people.

## One canonical model for every format

Dotenv, JSON, and TOML have different syntax and different type systems. The comparison engine shouldn't need three separate implementations to understand them.

Each parser normalizes input into `ConfigDocument` and `ConfigEntry`. The core compares paths and values without knowing which parser produced them.

## Preserve duplicates instead of silently picking a winner

A normal map would overwrite repeated keys. That's particularly unhelpful for dotenv and JSON, where duplicate assignments/object keys can otherwise disappear during parsing.

The model stores entries in a vector and tracks occurrence counts. The comparison layer can then report duplicates instead of pretending the input was unambiguous. TOML's parser rejects duplicate definitions as invalid TOML.

## Keep values out of diagnostics

The engine needs raw values to compare configurations, but users don't need secrets copied into CI logs to learn that two values differ.

Diagnostics contain a code, severity, path, source name, and optional location. They don't contain the compared values. `ConfigValue` also redacts its `Debug` output as a second line of defence.

## Stable output is a feature

Indexes use `BTreeMap`, and reports are sorted before they leave the core. This makes text and JSON output deterministic, which is useful for CI, snapshots, and humans trying to diff two runs.

## Keep parsing out of the core

The core doesn't open files, inspect extensions, or print to stdout. That makes the comparison logic easier to test and leaves room for other front ends without dragging CLI concerns into the library.

## Sensitivity is classification metadata

The core exposes `SensitivityPolicy` and per-entry `Sensitivity` metadata. The policy can classify paths and entries, but the built-in parser/CLI flow does not use those classifications when producing reports.

Reports omit configuration values for every entry, regardless of sensitivity. The classification API is available to callers, but it does not control CLI output. If EnvSentry gains a real use case for showing non-sensitive values, configurable redaction could be added to the reporting layer. See [Security](security.md).

## Locations are optional

`ConfigEntry` can carry a `SourceLocation`, and diagnostics preserve it when one is present. The built-in parsers don't attach locations, so normal CLI reports include the source filename but not line and column numbers. See [Diagnostics](diagnostics.md).
