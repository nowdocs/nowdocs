//! User-facing error taxonomy for CLI failures.

use std::fmt;

/// Error categories matching the spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Input,
    Network,
    Archive,
    Manifest,
    Cache,
    Model,
    Retrieval,
    Mcp,
    Internal,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input => write!(f, "input"),
            Self::Network => write!(f, "network"),
            Self::Archive => write!(f, "archive"),
            Self::Manifest => write!(f, "manifest"),
            Self::Cache => write!(f, "cache"),
            Self::Model => write!(f, "model"),
            Self::Retrieval => write!(f, "retrieval"),
            Self::Mcp => write!(f, "mcp"),
            Self::Internal => write!(f, "internal"),
        }
    }
}

/// Structured error for user-facing failures.
#[derive(Debug, Clone)]
pub struct NowdocsError {
    pub code: &'static str,
    pub category: ErrorCategory,
    pub message: String,
    pub hint: String,
}

impl fmt::Display for NowdocsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error[{}]: {}\nnext: {}",
            self.code, self.message, self.hint
        )
    }
}

impl std::error::Error for NowdocsError {}

/// Helper to create archive errors.
pub fn archive_error(
    code: &'static str,
    message: impl Into<String>,
    hint: impl Into<String>,
) -> NowdocsError {
    NowdocsError {
        code,
        category: ErrorCategory::Archive,
        message: message.into(),
        hint: hint.into(),
    }
}
