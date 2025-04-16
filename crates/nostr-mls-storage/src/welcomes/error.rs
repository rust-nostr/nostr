//! Error types for the welcomes module

use std::fmt;

/// Error types for the welcomes module
#[derive(Debug)]
pub enum WelcomeError {
    /// Invalid parameters
    InvalidParameters(String),
    /// Database error
    DatabaseError(String),
    /// Welcome not found
    NotFound,
}

impl std::error::Error for WelcomeError {}

impl fmt::Display for WelcomeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParameters(message) => write!(f, "Invalid parameters: {}", message),
            Self::DatabaseError(message) => write!(f, "Database error: {}", message),
            Self::NotFound => write!(f, "Welcome not found"),
        }
    }
}
