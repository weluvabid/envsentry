mod compare;
mod diagnostic;
mod error;
mod model;
mod sensitivity;

pub use compare::{ComparisonOptions, ComparisonReport, compare_documents};
pub use diagnostic::{Diagnostic, DiagnosticCode, Severity};
pub use error::CoreError;
pub use model::{
    ConfigDocument, ConfigEntry, ConfigFormat, ConfigPath, ConfigValue, PathSegment, Sensitivity,
    SourceLocation, ValueKind,
};
pub use sensitivity::SensitivityPolicy;
