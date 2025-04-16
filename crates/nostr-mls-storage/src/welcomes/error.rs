use std::fmt;

pub enum WelcomeError {
    InvalidParameters(String),
    NotFound,
    DatabaseError(String),
}

impl std::error::Error for WelcomeError {}

impl fmt::Display for WelcomeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParameters(message) => write!(f, "Invalid parameters: {}", message),
            Self::NotFound => write!(f, "Welcome not found"),
            Self::DatabaseError(message) => write!(f, "Database error: {}", message),
        }
    }
}

// TODO: derive Debug trait instead?
impl fmt::Debug for WelcomeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
