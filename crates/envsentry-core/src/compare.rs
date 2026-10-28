use std::collections::BTreeMap;

use crate::{ConfigDocument, ConfigEntry, ConfigPath, Diagnostic, DiagnosticCode, Severity};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComparisonOptions {
    compare_values: bool,
    report_unexpected_keys: bool,
}

impl Default for ComparisonOptions {
    fn default() -> Self {
        Self {
            compare_values: true,
            report_unexpected_keys: true,
        }
    }
}

impl ComparisonOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compare_values(mut self, enabled: bool) -> Self {
        self.compare_values = enabled;
        self
    }

    pub fn report_unexpected_keys(mut self, enabled: bool) -> Self {
        self.report_unexpected_keys = enabled;
        self
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ComparisonReport {
    diagnostics: Vec<Diagnostic>,
}

impl ComparisonReport {
    pub fn new(mut diagnostics: Vec<Diagnostic>) -> Self {
        diagnostics.sort_by(|left, right| {
            (
                left.path().map(ToString::to_string),
                left.code(),
                left.severity(),
                left.source_name(),
                left.location(),
            )
                .cmp(&(
                    right.path().map(ToString::to_string),
                    right.code(),
                    right.severity(),
                    right.source_name(),
                    right.location(),
                ))
        });
        Self { diagnostics }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity() == Severity::Error)
    }

    pub fn has_warnings(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity() == Severity::Warning)
    }

    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Compares `target` against `reference`. The reference defines the expected schema.
pub fn compare_documents(
    reference: &ConfigDocument,
    target: &ConfigDocument,
    options: ComparisonOptions,
) -> ComparisonReport {
    let mut diagnostics = Vec::new();

    let reference_index = index_entries(reference.entries());
    let target_index = index_entries(target.entries());

    add_duplicate_diagnostics(reference, &reference_index, &mut diagnostics);
    add_duplicate_diagnostics(target, &target_index, &mut diagnostics);

    for (path, entries) in &reference_index {
        if entries.len() == 1 && !target_index.contains_key(path) {
            diagnostics.push(diagnostic_for(
                DiagnosticCode::MissingKey,
                Severity::Error,
                path,
                reference,
                entries.first().copied(),
            ));
        }
    }

    if options.report_unexpected_keys {
        for (path, entries) in &target_index {
            if entries.len() == 1 && !reference_index.contains_key(path) {
                diagnostics.push(diagnostic_for(
                    DiagnosticCode::UnexpectedKey,
                    Severity::Warning,
                    path,
                    target,
                    entries.first().copied(),
                ));
            }
        }
    }

    for (path, entries) in &reference_index {
        let Some(target_entries) = target_index.get(path) else {
            continue;
        };

        if entries.len() != 1 || target_entries.len() != 1 {
            continue;
        }

        let reference_entry = &entries[0];
        let target_entry = &target_entries[0];

        if !options.compare_values {
            continue;
        }

        if reference_entry.value().kind() != target_entry.value().kind() {
            diagnostics.push(diagnostic_for(
                DiagnosticCode::TypeMismatch,
                Severity::Warning,
                path,
                target,
                Some(target_entry),
            ));
        } else if reference_entry.value().as_str() != target_entry.value().as_str() {
            diagnostics.push(diagnostic_for(
                DiagnosticCode::ValueMismatch,
                Severity::Warning,
                path,
                target,
                Some(target_entry),
            ));
        }
    }

    // Empty values must be reported even when the key is missing from the other side,
    // and duplicate entries must not disappear from analysis.
    add_all_empty_value_diagnostics(reference, &reference_index, &mut diagnostics);
    add_all_empty_value_diagnostics(target, &target_index, &mut diagnostics);

    ComparisonReport::new(diagnostics)
}

fn index_entries(entries: &[ConfigEntry]) -> BTreeMap<ConfigPath, Vec<&ConfigEntry>> {
    let mut index = BTreeMap::new();
    for entry in entries {
        index
            .entry(entry.path().clone())
            .or_insert_with(Vec::new)
            .push(entry);
    }
    index
}

fn add_duplicate_diagnostics(
    document: &ConfigDocument,
    index: &BTreeMap<ConfigPath, Vec<&ConfigEntry>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (path, entries) in index {
        for duplicate in entries.iter().skip(1) {
            diagnostics.push(diagnostic_for(
                DiagnosticCode::DuplicateKey,
                Severity::Error,
                path,
                document,
                Some(duplicate),
            ));
        }
    }
}

fn add_empty_value_diagnostic(
    document: &ConfigDocument,
    entry: &ConfigEntry,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if entry.value().is_empty() || entry.value().is_whitespace_only() {
        diagnostics.push(diagnostic_for(
            DiagnosticCode::EmptyValue,
            Severity::Warning,
            entry.path(),
            document,
            Some(entry),
        ));
    }
}

fn add_all_empty_value_diagnostics(
    document: &ConfigDocument,
    index: &BTreeMap<ConfigPath, Vec<&ConfigEntry>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for entries in index.values() {
        for entry in entries {
            add_empty_value_diagnostic(document, entry, diagnostics);
        }
    }
}

fn diagnostic_for(
    code: DiagnosticCode,
    severity: Severity,
    path: &ConfigPath,
    document: &ConfigDocument,
    entry: Option<&ConfigEntry>,
) -> Diagnostic {
    let mut diagnostic = Diagnostic::new(code, severity).with_path(path.clone());

    if let Some(source_name) = document.source_name() {
        diagnostic = diagnostic.with_source_name(source_name);
    }

    if let Some(location) = entry.and_then(|entry| entry.source()) {
        diagnostic = diagnostic.with_location(location);
    }

    diagnostic
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConfigEntry, ConfigFormat, ConfigValue, ValueKind};

    fn entry(key: &str, value: &str) -> ConfigEntry {
        ConfigEntry::new(
            ConfigPath::key(key),
            ConfigValue::new(value, ValueKind::String),
        )
    }

    fn document(entries: Vec<ConfigEntry>) -> ConfigDocument {
        ConfigDocument::new(ConfigFormat::Dotenv, entries)
    }

    fn codes(report: &ComparisonReport) -> Vec<DiagnosticCode> {
        report.diagnostics().iter().map(Diagnostic::code).collect()
    }

    #[test]
    fn detects_missing_unexpected_and_mismatched_values() {
        let reference = document(vec![
            entry("PORT", "3000"),
            entry("DATABASE_URL", "secret-a"),
        ]);
        let target = document(vec![entry("PORT", "4000"), entry("EXTRA", "value")]);

        let report = compare_documents(&reference, &target, ComparisonOptions::default());

        assert!(codes(&report).contains(&DiagnosticCode::MissingKey));
        assert!(codes(&report).contains(&DiagnosticCode::UnexpectedKey));
        assert!(codes(&report).contains(&DiagnosticCode::ValueMismatch));
    }

    #[test]
    fn detects_duplicates_in_both_documents() {
        let reference = document(vec![entry("PORT", "1"), entry("PORT", "2")]);
        let target = document(vec![entry("PORT", "3"), entry("PORT", "4")]);

        let report = compare_documents(&reference, &target, ComparisonOptions::default());

        assert_eq!(
            report
                .diagnostics()
                .iter()
                .filter(|diagnostic| diagnostic.code() == DiagnosticCode::DuplicateKey)
                .count(),
            2
        );
    }

    #[test]
    fn detects_empty_values_on_unmatched_and_duplicate_entries() {
        let reference = document(vec![entry("EMPTY", "")]);
        let target = document(vec![entry("OTHER", "x"), entry("OTHER", " ")]);

        let report = compare_documents(&reference, &target, ComparisonOptions::default());

        assert_eq!(
            report
                .diagnostics()
                .iter()
                .filter(|diagnostic| diagnostic.code() == DiagnosticCode::EmptyValue)
                .count(),
            2
        );
    }

    #[test]
    fn does_not_compare_values_when_schema_only_is_selected() {
        let reference = document(vec![entry("PORT", "3000")]);
        let target = document(vec![entry("PORT", "4000")]);

        let report = compare_documents(
            &reference,
            &target,
            ComparisonOptions::new().compare_values(false),
        );

        assert!(report.is_clean());
    }

    #[test]
    fn report_order_is_deterministic() {
        let reference = document(vec![entry("ZED", "1"), entry("ALPHA", "1")]);
        let target = document(vec![entry("ZED", "2"), entry("ALPHA", "2")]);

        let first = compare_documents(&reference, &target, ComparisonOptions::default());
        let second = compare_documents(&reference, &target, ComparisonOptions::default());

        assert_eq!(first, second);
        assert_eq!(first.diagnostics()[0].path().unwrap().to_string(), "ALPHA");
    }

    #[test]
    fn diagnostics_never_include_configuration_values() {
        let secret = "SENTRY_SECRET_SENTINEL";
        let reference = document(vec![entry("API_TOKEN", secret)]);
        let target = document(vec![entry("API_TOKEN", "other-secret")]);

        let report = compare_documents(&reference, &target, ComparisonOptions::default());
        let rendered = format!("{report:?}");

        assert!(!rendered.contains(secret));
        assert!(!rendered.contains("other-secret"));
    }
}
