use crate::{ConfigEntry, ConfigPath, PathSegment, Sensitivity};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SensitivityPolicy {
    sensitive_patterns: Vec<String>,
    public_patterns: Vec<String>,
    redact_all: bool,
}

impl Default for SensitivityPolicy {
    fn default() -> Self {
        Self {
            sensitive_patterns: vec![
                "*PASSWORD*".into(),
                "*PASSWD*".into(),
                "*SECRET*".into(),
                "*TOKEN*".into(),
                "*API_KEY*".into(),
                "*PRIVATE_KEY*".into(),
                "*CREDENTIAL*".into(),
                "*DATABASE_URL*".into(),
                "*CONNECTION_STRING*".into(),
                "*CLIENT_SECRET*".into(),
                "*ACCESS_KEY*".into(),
            ],
            public_patterns: Vec::new(),
            redact_all: false,
        }
    }
}

impl SensitivityPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_sensitive_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.sensitive_patterns.push(pattern.into().to_ascii_uppercase());
        self
    }

    pub fn with_public_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.public_patterns.push(pattern.into().to_ascii_uppercase());
        self
    }

    pub fn redact_all(mut self, redact_all: bool) -> Self {
        self.redact_all = redact_all;
        self
    }

    pub fn redacts_all(&self) -> bool {
        self.redact_all
    }

    pub fn classify_path(&self, path: &ConfigPath) -> Sensitivity {
        if self.redact_all {
            return Sensitivity::Sensitive;
        }

        let key = path_key(path).to_ascii_uppercase();

        if matches_any(&key, &self.public_patterns) {
            return Sensitivity::Public;
        }
        if matches_any(&key, &self.sensitive_patterns) {
            return Sensitivity::Sensitive;
        }

        Sensitivity::Unknown
    }

    pub fn classify_entry(&self, entry: &ConfigEntry) -> Sensitivity {
        if self.redact_all {
            return Sensitivity::Sensitive;
        }

        match entry.sensitivity() {
            Sensitivity::Sensitive => Sensitivity::Sensitive,
            Sensitivity::Public => Sensitivity::Public,
            Sensitivity::Unknown => self.classify_path(entry.path()),
        }
    }
}

fn path_key(path: &ConfigPath) -> String {
    path.segments()
        .iter()
        .filter_map(|segment| match segment {
            PathSegment::Key(key) => Some(key.as_str()),
            PathSegment::Index(_) => None,
        })
        .collect::<Vec<_>>()
        .join(".")
}

fn matches_any(value: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|pattern| wildcard_match(pattern, value))
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    let parts: Vec<_> = pattern.split('*').filter(|part| !part.is_empty()).collect();

    if parts.is_empty() {
        return true;
    }

    let mut cursor = 0;
    let anchored_start = !pattern.starts_with('*');
    let anchored_end = !pattern.ends_with('*');

    for (index, part) in parts.iter().enumerate() {
        let Some(found) = value[cursor..].find(part) else {
            return false;
        };

        if index == 0 && anchored_start && found != 0 {
            return false;
        }

        cursor += found + part.len();
    }

    !anchored_end || cursor == value.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_common_sensitive_names_case_insensitively() {
        let policy = SensitivityPolicy::default();

        assert_eq!(policy.classify_path(&ConfigPath::key("database_url")), Sensitivity::Sensitive);
        assert_eq!(policy.classify_path(&ConfigPath::key("SERVICE_TOKEN")), Sensitivity::Sensitive);
        assert_eq!(policy.classify_path(&ConfigPath::key("LOG_LEVEL")), Sensitivity::Unknown);
    }

    #[test]
    fn explicit_public_patterns_take_precedence_over_defaults() {
        let policy = SensitivityPolicy::default().with_public_pattern("PUBLIC_*");

        assert_eq!(policy.classify_path(&ConfigPath::key("PUBLIC_API_KEY")), Sensitivity::Public);
    }

    #[test]
    fn explicit_entry_classification_takes_precedence() {
        let policy = SensitivityPolicy::default();
        let entry = ConfigEntry::new(
            ConfigPath::key("LOG_LEVEL"),
            crate::ConfigValue::new("debug", crate::ValueKind::String),
        )
        .with_sensitivity(Sensitivity::Sensitive);

        assert_eq!(policy.classify_entry(&entry), Sensitivity::Sensitive);
    }

    #[test]
    fn redact_all_overrides_explicit_public_classification() {
        let policy = SensitivityPolicy::default().redact_all(true);
        let entry = ConfigEntry::new(
            ConfigPath::key("LOG_LEVEL"),
            crate::ConfigValue::new("debug", crate::ValueKind::String),
        )
        .with_sensitivity(Sensitivity::Public);

        assert_eq!(policy.classify_path(&ConfigPath::key("LOG_LEVEL")), Sensitivity::Sensitive);
        assert_eq!(policy.classify_entry(&entry), Sensitivity::Sensitive);
    }
}
