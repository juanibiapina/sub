use std::fmt;

#[derive(Debug, Clone)]
pub enum MagicError {
    InvalidUsageString(String),
    InvalidOptionString(String),
    InvalidUTF8,
    IoError(String),
    ParseError(String),
}

impl fmt::Display for MagicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MagicError::InvalidUsageString(msg) => write!(f, "Invalid usage string: {}", msg),
            MagicError::InvalidOptionString(msg) => write!(f, "Invalid option string: {}", msg),
            MagicError::InvalidUTF8 => write!(f, "Invalid UTF-8 in file"),
            MagicError::IoError(msg) => write!(f, "IO error: {}", msg),
            MagicError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for MagicError {}

pub type Result<T> = std::result::Result<T, MagicError>;