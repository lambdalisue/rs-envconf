//! Environment variable configuration with file-based secrets support
//!
//! `serviceconf` provides a declarative API for loading configuration from environment
//! variables with native support for **file-based secrets** (Kubernetes Secrets, Docker Secrets).
//!
//! The primary feature that distinguishes this library from other environment variable
//! configuration libraries is the `#[conf(from_file)]` attribute, which allows reading
//! secrets from files while falling back to direct environment variables for local development.
//!
//! # Features
//!
//! - **File-based secrets**: Read secrets from Kubernetes/Docker mounted files
//! - **Declarative**: Automatic implementation with `#[derive(ServiceConf)]`
//! - **Type-safe**: Compile-time type checking
//! - **Default values**: Support for `Default` trait and explicit values
//! - **Custom deserializers**: Support for JSON, TOML, or custom parsing functions
//!
//! # Value Parsing
//!
//! **Default (using `FromStr`)**:
//! - Strings: `DATABASE_URL=postgres://localhost/db`
//! - Numbers: `MAX_CONNECTIONS=42`
//! - Booleans: `DEBUG=true`
//!
//! **Custom deserializers** - specify with `#[conf(deserializer = "function")]`:
//! - JSON: `#[conf(deserializer = "serde_json::from_str")]`
//! - TOML: `#[conf(deserializer = "toml::from_str")]`
//! - Custom: Define your own deserializer function
//!
//! # Example
//!
//! ```rust
//! use serviceconf::ServiceConf;
//!
//! #[derive(Debug, ServiceConf)]
//! struct Config {
//!     // File-based secret: reads from API_KEY or API_KEY_FILE
//!     #[conf(from_file)]
//!     pub api_key: String,
//!
//!     // Default value if not set
//!     #[conf(default = 8080)]
//!     pub port: u16,
//! }
//!
//! # fn main() -> anyhow::Result<()> {
//! #     std::env::set_var("API_KEY", "test-key");
//! #     let config = Config::from_env()?;
//! #     assert_eq!(config.api_key, "test-key");
//! #     assert_eq!(config.port, 8080);
//! #     Ok(())
//! # }
//! ```
//!
//! # Attributes
//!
//! ## `#[conf(from_file)]` - File-based Secrets
//!
//! Load from `{VAR_NAME}_FILE` in addition to the environment variable.
//! This is the primary feature of `serviceconf` for handling file-based secrets
//! in Kubernetes and Docker environments.
//!
//! **Loading priority:**
//! 1. Direct env var (`API_KEY`) - for local development
//! 2. File path from env var (`API_KEY_FILE`) - for production
//!
//! ```rust
//! # use serviceconf::ServiceConf;
//! #[derive(ServiceConf)]
//! pub struct Config {
//!     // Reads from API_KEY or API_KEY_FILE
//!     #[conf(from_file)]
//!     pub api_key: String,
//! }
//! ```
//!
//! ## `#[conf(default = "value")]`
//!
//! Specify a default value when the environment variable is not set.
//!
//! ```rust
//! # use serviceconf::ServiceConf;
//! #[derive(ServiceConf)]
//! struct Config {
//!     #[conf(default = "127.0.0.1:8080".to_string())]
//!     pub server_addr: String,
//!
//!     #[conf(default = 10)]
//!     pub max_connections: u32,
//!
//!     #[conf(default = false)]
//!     pub enable_tls: bool,
//! }
//! # fn main() -> anyhow::Result<()> {
//! #     let config = Config::from_env()?;
//! #     assert_eq!(config.max_connections, 10);
//! #     Ok(())
//! # }
//! ```
//!
//! ## `#[conf(name = "CUSTOM_NAME")]`
//!
//! Specify an environment variable name different from the field name.
//!
//! ```rust
//! # use serviceconf::ServiceConf;
//! #[derive(ServiceConf)]
//! pub struct Config {
//!     // Load from REDIS_URL environment variable
//!     #[conf(name = "REDIS_URL")]
//!     pub redis_connection_string: String,
//! }
//! ```

#[doc(hidden)]
pub mod de;

mod error;

pub use error::EnvError;
pub use serviceconf_derive::ServiceConf;

// Re-export for macro-generated code
#[doc(hidden)]
pub use anyhow;
