use std::fmt;

pub enum MessageError {
    InvalidParameters(String),
    NotFound,
    DatabaseError(String),
}

impl std::error::Error for MessageError {}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParameters(message) => write!(f, "Invalid parameters: {}", message),
            Self::NotFound => write!(f, "Message not found"),
            Self::DatabaseError(message) => write!(f, "Database error: {}", message),
        }
    }
}

impl fmt::Debug for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
