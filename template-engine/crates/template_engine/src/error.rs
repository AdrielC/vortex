use std::fmt;

#[derive(Debug)]
pub enum EngineError {
    InvalidArgument(String),
    Serde(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::InvalidArgument(message) => write!(f, "{message}"),
            EngineError::Serde(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<serde_json::Error> for EngineError {
    fn from(error: serde_json::Error) -> Self {
        EngineError::Serde(error.to_string())
    }
}
