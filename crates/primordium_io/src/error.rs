//! Error types for primordium_io crate.
//!
//! Provides structured error handling for all I/O operations including
//! persistence, networking, and storage.

use thiserror::Error;

/// Main error type for primordium_io operations.
#[derive(Error, Debug)]
pub enum IoError {
    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Rkyv-specific errors
    #[error("Rkyv error: {0}")]
    Rkyv(String),

    /// File system errors
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    /// JSON parsing errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Database errors
    #[error("Database error: {0}")]
    Database(String),

    /// Network errors
    #[error("Network error: {0}")]
    Network(String),

    /// Compression errors
    #[error("Compression error: {0}")]
    Compression(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Not found errors
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Permission errors
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Generic error with context
    #[error("{context}: {source}")]
    Context {
        context: String,
        source: Box<IoError>,
    },
}

/// Result type alias for primordium_io operations.
pub type Result<T> = std::result::Result<T, IoError>;

impl IoError {
    /// Creates a new serialization error.
    #[must_use]
    pub fn serialization<S: Into<String>>(msg: S) -> Self {
        Self::Serialization(msg.into())
    }

    /// Creates a new Rkyv error.
    #[must_use]
    pub fn rkyv<S: Into<String>>(msg: S) -> Self {
        Self::Rkyv(msg.into())
    }

    /// Creates a new network error.
    #[must_use]
    pub fn network<S: Into<String>>(msg: S) -> Self {
        Self::Network(msg.into())
    }

    /// Creates a new database error.
    #[must_use]
    pub fn database<S: Into<String>>(msg: S) -> Self {
        Self::Database(msg.into())
    }

    /// Creates a new validation error.
    #[must_use]
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    /// Creates a new not found error.
    #[must_use]
    pub fn not_found<S: Into<String>>(resource: S) -> Self {
        Self::NotFound(resource.into())
    }

    /// Creates a new compression error.
    #[must_use]
    pub fn compression<S: Into<String>>(msg: S) -> Self {
        Self::Compression(msg.into())
    }

    /// Wraps an error with additional context.
    #[must_use]
    pub fn with_context<S: Into<String>>(self, context: S) -> Self {
        Self::Context {
            context: context.into(),
            source: Box::new(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = IoError::serialization("test error");
        assert_eq!(err.to_string(), "Serialization error: test error");
    }

    #[test]
    fn test_error_context() {
        let err = IoError::not_found("config.toml").with_context("loading config");
        assert!(err.to_string().contains("loading config"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: IoError = io_err.into();
        matches!(err, IoError::FileSystem(_));
    }
}
