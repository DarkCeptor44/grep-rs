use std::fmt::Display;

pub(crate) type Result<T> = std::result::Result<T, GrepError>;

/// Grep errors
#[derive(Debug)]
pub(crate) enum GrepError {
    Io(String),
}

impl std::error::Error for GrepError {}

impl Display for GrepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrepError::Io(e) => write!(f, "grep-rs: {e}"),
        }
    }
}

impl From<std::io::Error> for GrepError {
    fn from(value: std::io::Error) -> Self {
        GrepError::Io(value.to_string())
    }
}
