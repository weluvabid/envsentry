use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ConfigFormat {
    Dotenv,
    Json,
    Toml,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConfigPath(Vec<PathSegment>);

impl ConfigPath {
    pub fn root() -> Self {
        Self::default()
    }

    pub fn from_segments(segments: impl IntoIterator<Item = PathSegment>) -> Self {
        Self(segments.into_iter().collect())
    }

    pub fn key(key: impl Into<String>) -> Self {
        Self(vec![PathSegment::Key(key.into())])
    }

    pub fn segments(&self) -> &[PathSegment] {
        &self.0
    }

    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    pub fn child_key(&self, key: impl Into<String>) -> Self {
        let mut segments = self.0.clone();
        segments.push(PathSegment::Key(key.into()));
        Self(segments)
    }

    pub fn child_index(&self, index: usize) -> Self {
        let mut segments = self.0.clone();
        segments.push(PathSegment::Index(index));
        Self(segments)
    }
}

impl fmt::Display for ConfigPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first_key = true;

        for segment in &self.0 {
            match segment {
                PathSegment::Key(key) => {
                    if !first_key {
                        f.write_str(".")?;
                    }
                    for character in key.chars() {
                        if matches!(character, '.' | '\\' | '[' | ']') {
                            f.write_str("\\")?;
                            write!(f, "{character}")?;
                        } else if character.is_control() {
                            write!(f, "{}", character.escape_default())?;
                        } else {
                            write!(f, "{character}")?;
                        }
                    }
                    first_key = false;
                }
                PathSegment::Index(index) => write!(f, "[{index}]")?,
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ValueKind {
    String,
    Integer,
    Float,
    Boolean,
    Null,
    Array,
    Object,
    DateTime,
    Unknown,
}

#[derive(Clone, Eq, PartialEq)]
pub struct ConfigValue {
    raw: String,
    kind: ValueKind,
}

impl ConfigValue {
    pub fn new(raw: impl Into<String>, kind: ValueKind) -> Self {
        Self { raw: raw.into(), kind }
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }

    pub fn kind(&self) -> ValueKind {
        self.kind
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    pub fn is_whitespace_only(&self) -> bool {
        !self.raw.is_empty() && self.raw.chars().all(char::is_whitespace)
    }
}

impl fmt::Debug for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigValue").field("raw", &"[REDACTED]").field("kind", &self.kind).finish()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Sensitivity {
    Public,
    Sensitive,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceLocation {
    line: usize,
    column: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Option<Self> {
        if line == 0 || column == 0 {
            return None;
        }
        Some(Self { line, column })
    }

    pub fn line(self) -> usize {
        self.line
    }

    pub fn column(self) -> usize {
        self.column
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigEntry {
    path: ConfigPath,
    value: ConfigValue,
    sensitivity: Sensitivity,
    source: Option<SourceLocation>,
    occurrence: usize,
}

impl ConfigEntry {
    pub fn new(path: ConfigPath, value: ConfigValue) -> Self {
        Self { path, value, sensitivity: Sensitivity::Unknown, source: None, occurrence: 1 }
    }

    pub fn with_sensitivity(mut self, sensitivity: Sensitivity) -> Self {
        self.sensitivity = sensitivity;
        self
    }

    pub fn with_source(mut self, source: SourceLocation) -> Self {
        self.source = Some(source);
        self
    }

    pub fn with_occurrence(mut self, occurrence: usize) -> Self {
        self.occurrence = occurrence.max(1);
        self
    }

    pub fn path(&self) -> &ConfigPath {
        &self.path
    }

    pub fn value(&self) -> &ConfigValue {
        &self.value
    }

    pub fn sensitivity(&self) -> Sensitivity {
        self.sensitivity
    }

    pub fn source(&self) -> Option<SourceLocation> {
        self.source
    }

    pub fn occurrence(&self) -> usize {
        self.occurrence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigDocument {
    format: ConfigFormat,
    source_name: Option<String>,
    entries: Vec<ConfigEntry>,
}

impl ConfigDocument {
    pub fn new(format: ConfigFormat, entries: Vec<ConfigEntry>) -> Self {
        Self { format, source_name: None, entries }
    }

    pub fn with_source_name(mut self, source_name: impl Into<String>) -> Self {
        self.source_name = Some(source_name.into());
        self
    }

    pub fn format(&self) -> ConfigFormat {
        self.format
    }

    pub fn source_name(&self) -> Option<&str> {
        self.source_name.as_deref()
    }

    pub fn entries(&self) -> &[ConfigEntry] {
        &self.entries
    }

    pub fn into_entries(self) -> Vec<ConfigEntry> {
        self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_debug_redacts_raw_contents() {
        let value = ConfigValue::new("SENTRY_TEST_SECRET_7f31c2", ValueKind::String);
        let debug = format!("{value:?}");

        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("SENTRY_TEST_SECRET_7f31c2"));
    }

    #[test]
    fn empty_and_whitespace_only_values_are_distinct() {
        let empty = ConfigValue::new("", ValueKind::String);
        let whitespace = ConfigValue::new("  \t", ValueKind::String);

        assert!(empty.is_empty());
        assert!(!empty.is_whitespace_only());
        assert!(!whitespace.is_empty());
        assert!(whitespace.is_whitespace_only());
    }

    #[test]
    fn config_path_formats_nested_keys_and_indices() {
        let path = ConfigPath::key("services").child_index(1).child_key("api.url");

        assert_eq!(path.to_string(), "services[1].api\\.url");
    }

    #[test]
    fn config_path_escapes_control_characters() {
        let path = ConfigPath::key("API\nKEY").child_key("ESC\u{1b}APE");

        assert_eq!(path.to_string(), "API\\nKEY.ESC\\u{1b}APE");
        assert!(!path.to_string().contains('\n'));
        assert!(!path.to_string().contains('\u{1b}'));
    }

    #[test]
    fn config_path_escapes_array_delimiters_in_keys() {
        let key_path = ConfigPath::key("items[0]");
        let index_path = ConfigPath::key("items").child_index(0);

        assert_eq!(key_path.to_string(), "items\\[0\\]");
        assert_eq!(index_path.to_string(), "items[0]");
        assert_ne!(key_path.to_string(), index_path.to_string());
    }

    #[test]
    fn source_locations_reject_zero_coordinates() {
        assert!(SourceLocation::new(0, 1).is_none());
        assert!(SourceLocation::new(1, 0).is_none());
        assert_eq!(SourceLocation::new(2, 3).unwrap().line(), 2);
    }

    #[test]
    fn document_keeps_duplicate_entries_in_order() {
        let first =
            ConfigEntry::new(ConfigPath::key("PORT"), ConfigValue::new("3000", ValueKind::String))
                .with_occurrence(1);
        let second =
            ConfigEntry::new(ConfigPath::key("PORT"), ConfigValue::new("4000", ValueKind::String))
                .with_occurrence(2);
        let document = ConfigDocument::new(ConfigFormat::Dotenv, vec![first, second]);

        assert_eq!(document.entries().len(), 2);
        assert_eq!(document.entries()[1].occurrence(), 2);
    }
}
