use crate::{ConfigPath, SourceLocation};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DiagnosticCode {
    MissingKey,
    UnexpectedKey,
    DuplicateKey,
    EmptyValue,
    ValueMismatch,
    TypeMismatch,
    ParseError,
}

impl DiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingKey => "MISSING_KEY",
            Self::UnexpectedKey => "UNEXPECTED_KEY",
            Self::DuplicateKey => "DUPLICATE_KEY",
            Self::EmptyValue => "EMPTY_VALUE",
            Self::ValueMismatch => "VALUE_MISMATCH",
            Self::TypeMismatch => "TYPE_MISMATCH",
            Self::ParseError => "PARSE_ERROR",
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    code: DiagnosticCode,
    severity: Severity,
    path: Option<ConfigPath>,
    source_name: Option<String>,
    location: Option<SourceLocation>,
}

impl Diagnostic {
    pub fn new(code: DiagnosticCode, severity: Severity) -> Self {
        Self { code, severity, path: None, source_name: None, location: None }
    }

    pub fn with_path(mut self, path: ConfigPath) -> Self {
        self.path = Some(path);
        self
    }

    pub fn with_source_name(mut self, source_name: impl Into<String>) -> Self {
        self.source_name = Some(source_name.into());
        self
    }

    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    pub fn code(&self) -> DiagnosticCode {
        self.code
    }

    pub fn severity(&self) -> Severity {
        self.severity
    }

    pub fn path(&self) -> Option<&ConfigPath> {
        self.path.as_ref()
    }

    pub fn source_name(&self) -> Option<&str> {
        self.source_name.as_deref()
    }

    pub fn location(&self) -> Option<SourceLocation> {
        self.location
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_codes_have_stable_names() {
        assert_eq!(DiagnosticCode::MissingKey.as_str(), "MISSING_KEY");
        assert_eq!(DiagnosticCode::ParseError.to_string(), "PARSE_ERROR");
    }
}
