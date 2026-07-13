use thiserror::Error;

/// Errors produced by domain validation or persistence ports.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// A required configuration value is invalid.
    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),
    /// A conversation message is empty after trimming.
    #[error("message cannot be empty")]
    EmptyMessage,
    /// Persisted data uses an unsupported schema.
    #[error("unsupported memory schema version {found}; expected {expected}")]
    UnsupportedSchema {
        /// Version found in persisted data.
        found: u32,
        /// Version supported by this build.
        expected: u32,
    },
    /// A persistence adapter failed.
    #[error("storage failure: {0}")]
    Storage(String),
}
