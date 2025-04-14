use std::fmt;

pub enum GroupError {
    InvalidParameters(String),
    NotFound,
    DatabaseError(String),
}

impl std::error::Error for GroupError {}

impl fmt::Display for GroupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParameters(message) => write!(f, "Invalid parameters: {}", message),
            Self::NotFound => write!(f, "Group not found"),
            Self::DatabaseError(message) => write!(f, "Database error: {}", message),
        }
    }
}

impl fmt::Debug for GroupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
