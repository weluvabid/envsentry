mod diagnostic;
mod error;
mod model;

pub use diagnostic::{Diagnostic, DiagnosticCode, Severity};
pub use error::CoreError;
pub use model::{
    ConfigDocument, ConfigEntry, ConfigFormat, ConfigPath, ConfigValue, PathSegment, Sensitivity,
    SourceLocation, ValueKind,
};
