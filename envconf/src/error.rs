//! Error types for environment variable configuration

/// Environment variable loading error
#[derive(Debug, thiserror::Error)]
pub enum EnvError {
    /// Environment variable not found
    #[error("Environment variable '{name}' is required but not set")]
    Missing { name: String },

    /// File read error
    #[error("Failed to read file '{path}' for environment variable '{name}': {source}")]
    FileRead {
        name: String,
        path: String,
        source: std::io::Error,
    },

    /// Parse error
    #[error("Failed to parse environment variable '{name}' as {type_name}: {message}")]
    Parse {
        name: String,
        type_name: String,
        message: String,
    },
}

impl EnvError {
    /// Create a parse error (used by macro-generated code)
    #[doc(hidden)]
    pub fn parse_error<T>(name: impl Into<String>, message: impl std::fmt::Display) -> Self {
        Self::Parse {
            name: name.into(),
            type_name: std::any::type_name::<T>().to_string(),
            message: message.to_string(),
        }
    }

    /// Create a missing environment variable error (used by macro-generated code)
    #[doc(hidden)]
    pub fn missing(name: impl Into<String>) -> Self {
        Self::Missing { name: name.into() }
    }
}
