use std::{error::Error, fmt};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreError {
    InvalidConfiguration,
    IncompatibleDocuments,
    InvalidRuleConfiguration,
    InternalInvariant,
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidConfiguration => "invalid configuration",
            Self::IncompatibleDocuments => "incompatible configuration documents",
            Self::InvalidRuleConfiguration => "invalid rule configuration",
            Self::InternalInvariant => "internal invariant violated",
        };
        f.write_str(message)
    }
}

impl Error for CoreError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_use_safe_generic_messages() {
        let message = CoreError::InvalidConfiguration.to_string();

        assert_eq!(message, "invalid configuration");
        assert!(!message.contains("password"));
    }
}
