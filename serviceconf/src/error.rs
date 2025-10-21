//! Error types for environment variable configuration

/// Errors that can occur when loading configuration from environment variables.
///
/// This error type covers three main failure scenarios:
/// - Missing required environment variables
/// - File read failures when using the `{VAR}_FILE` pattern
/// - Type parsing failures during deserialization
#[derive(Debug, thiserror::Error)]
pub enum ServiceConfError {
    /// Required environment variable is not set.
    ///
    /// Occurs when a non-optional field's environment variable is not found
    /// and no default value is specified.
    #[error("Environment variable '{name}' is required but not set")]
    Missing {
        /// Name of the missing environment variable
        name: String,
    },

    /// Failed to read from a file specified by `{VAR}_FILE` environment variable.
    ///
    /// When using `#[conf(from_file)]`, this error occurs if the file path
    /// specified in `{VAR}_FILE` cannot be read (e.g., file doesn't exist,
    /// permission denied).
    #[error("Failed to read file '{path}' for environment variable '{name}': {source}")]
    FileRead {
        /// Name of the `{VAR}_FILE` environment variable (e.g., "API_KEY_FILE")
        name: String,
        /// Path to the file that failed to be read
        path: String,
        /// Underlying I/O error that caused the failure
        source: std::io::Error,
    },

    /// Failed to parse environment variable value into the target type.
    ///
    /// Occurs when the string value cannot be converted to the field's type,
    /// either via `FromStr` or a custom deserializer function.
    #[error("Failed to parse environment variable '{name}' as {type_name}: {message}")]
    Parse {
        /// Name of the environment variable being parsed
        name: String,
        /// Fully qualified type name that parsing was attempted for
        type_name: String,
        /// Error message from the parser (FromStr or custom deserializer)
        message: String,
    },
}

impl ServiceConfError {
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
