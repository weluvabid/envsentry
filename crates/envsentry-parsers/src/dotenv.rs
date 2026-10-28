use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::Cursor;

use envsentry_core::{
    ConfigDocument, ConfigEntry, ConfigFormat, ConfigPath, ConfigValue, Sensitivity, ValueKind,
};

/// A safe, value-free error produced while parsing dotenv input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DotenvParseError;

impl fmt::Display for DotenvParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid dotenv syntax")
    }
}

impl Error for DotenvParseError {}

/// Parses dotenv input without mutating the process environment.
///
/// Syntax handling is delegated to `dotenvy`. Entries are consumed from its
/// iterator so repeated assignments remain visible to EnvSentry.
pub fn parse_dotenv(input: &str) -> Result<ConfigDocument, DotenvParseError> {
    let mut entries = Vec::new();
    let mut occurrences = HashMap::<String, usize>::new();

    for item in dotenvy::from_read_iter(Cursor::new(input.as_bytes())) {
        let (key, value) = item.map_err(|_| DotenvParseError)?;
        let occurrence = occurrences.entry(key.clone()).or_insert(0);
        *occurrence += 1;

        entries.push(
            ConfigEntry::new(ConfigPath::key(key), ConfigValue::new(value, ValueKind::String))
                .with_sensitivity(Sensitivity::Unknown)
                .with_occurrence(*occurrence),
        );
    }

    Ok(ConfigDocument::new(ConfigFormat::Dotenv, entries))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_dotenv_syntax() {
        let document = parse_dotenv(
            "# comment\nPORT=3000\nexport APP_NAME=envsentry\nEMPTY=\nA='hello # world'\nB=\"line\\nnext\"\n",
        )
        .unwrap();

        assert_eq!(document.entries().len(), 5);
        assert_eq!(document.entries()[0].path().to_string(), "PORT");
        assert_eq!(document.entries()[0].value().as_str(), "3000");
        assert_eq!(document.entries()[1].value().as_str(), "envsentry");
        assert_eq!(document.entries()[2].value().as_str(), "");
        assert_eq!(document.entries()[3].value().as_str(), "hello # world");
        assert_eq!(document.entries()[4].value().as_str(), "line\nnext");
    }

    #[test]
    fn preserves_duplicate_assignments() {
        let document = parse_dotenv("PORT=3000\nPORT=4000\nPORT=5000\n").unwrap();

        assert_eq!(document.entries().len(), 3);
        assert_eq!(document.entries()[0].occurrence(), 1);
        assert_eq!(document.entries()[1].occurrence(), 2);
        assert_eq!(document.entries()[2].occurrence(), 3);
        assert_eq!(document.entries()[1].value().as_str(), "4000");
    }

    #[test]
    fn parser_errors_do_not_include_input_or_values() {
        let secret = "PASSWORD='SENTRY_SECRET_SENTINEL";
        let error = parse_dotenv(secret).unwrap_err().to_string();

        assert_eq!(error, "invalid dotenv syntax");
        assert!(!error.contains("PASSWORD"));
        assert!(!error.contains("SENTRY_SECRET_SENTINEL"));
    }

    #[test]
    fn empty_and_whitespace_values_remain_distinct() {
        let document = parse_dotenv("EMPTY=\nSPACES=\"  \"\n").unwrap();

        assert!(document.entries()[0].value().is_empty());
        assert!(document.entries()[1].value().is_whitespace_only());
    }

    #[test]
    fn parsing_does_not_load_values_into_process_environment() {
        let key = "ENVSENTRY_TEST_SHOULD_NOT_BE_SET_84721";
        let document = parse_dotenv(&format!("{key}=sentinel")).unwrap();

        assert_eq!(document.entries().len(), 1);
        assert!(std::env::var_os(key).is_none());
    }
}
