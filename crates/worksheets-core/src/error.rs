//! Error types for `worksheets-core`.

use thiserror::Error;

/// Errors returned by worksheet validation, semantic parsing, or action execution.
#[derive(Debug, Error)]
pub enum WorksheetError {
    /// Field value failed type or enum validation.
    #[error("validation error on field `{field}`: {reason}")]
    Validation {
        /// Field name.
        field: String,
        /// Failure reason.
        reason: String,
    },

    /// Referenced worksheet or field does not exist.
    #[error("unknown symbol: {0}")]
    Unknown(String),

    /// Worksheet action callback failed.
    #[error("action execution error: {0}")]
    Action(String),
}
