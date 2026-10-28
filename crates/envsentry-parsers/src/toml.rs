use std::fmt;

use envsentry_core::{
    ConfigDocument, ConfigEntry, ConfigFormat, ConfigPath, ConfigValue, Sensitivity, ValueKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TomlParseError;

impl fmt::Display for TomlParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid TOML configuration")
    }
}

impl std::error::Error for TomlParseError {}

/// Parses TOML into the canonical flattened path representation.
pub fn parse_toml(input: &str) -> Result<ConfigDocument, TomlParseError> {
    let value: toml::Value = toml::from_str(input).map_err(|_| TomlParseError)?;
    let mut entries = Vec::new();
    flatten(value, &ConfigPath::root(), &mut entries);

    Ok(ConfigDocument::new(ConfigFormat::Toml, entries))
}

fn flatten(value: toml::Value, path: &ConfigPath, entries: &mut Vec<ConfigEntry>) {
    match value {
        toml::Value::Table(table) => {
            for (key, value) in table {
                flatten(value, &path.child_key(key), entries);
            }
        }
        toml::Value::Array(values) => {
            for (index, value) in values.into_iter().enumerate() {
                flatten(value, &path.child_index(index), entries);
            }
        }
        leaf => {
            let (raw, kind) = match leaf {
                toml::Value::String(value) => (value, ValueKind::String),
                toml::Value::Integer(value) => (value.to_string(), ValueKind::Integer),
                toml::Value::Float(value) => (value.to_string(), ValueKind::Float),
                toml::Value::Boolean(value) => (value.to_string(), ValueKind::Boolean),
                toml::Value::Datetime(value) => (value.to_string(), ValueKind::DateTime),
                toml::Value::Array(_) | toml::Value::Table(_) => unreachable!(),
            };
            entries.push(
                ConfigEntry::new(path.clone(), ConfigValue::new(raw, kind))
                    .with_sensitivity(Sensitivity::Unknown),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flattens_tables_and_arrays_with_native_types() {
        let document = parse_toml(
            r#"[database]
                    port = 5432
                    enabled = true
                    tags = ["primary", "replica"]"#,
        )
        .unwrap();

        let paths: Vec<_> = document
            .entries()
            .iter()
            .map(|entry| entry.path().to_string())
            .collect();

        assert!(paths.contains(&"database.port".to_owned()));
        assert!(paths.contains(&"database.enabled".to_owned()));
        assert!(paths.contains(&"database.tags[0]".to_owned()));
        assert_eq!(
            document
                .entries()
                .iter()
                .find(|entry| entry.path().to_string() == "database.port")
                .unwrap()
                .value()
                .kind(),
            ValueKind::Integer
        );
    }

    #[test]
    fn duplicate_definitions_are_rejected() {
        let error = parse_toml("port = 3000\nport = 4000\n").unwrap_err();
        assert_eq!(error.to_string(), "invalid TOML configuration");
    }

    #[test]
    fn parse_errors_do_not_echo_secret_values() {
        let error = parse_toml("password = [\"SENSITIVE_SENTINEL\"")
            .unwrap_err()
            .to_string();

        assert_eq!(error, "invalid TOML configuration");
        assert!(!error.contains("SENSITIVE_SENTINEL"));
    }
}
