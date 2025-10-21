//! Environment variable-based configuration management library
//!
//! This library provides a declarative API for loading configuration
//! from environment variables into structs.
//!
//! # Features
//!
//! - **Declarative**: Automatic implementation with `#[derive(EnvConf)]`
//! - **File-based configuration**: Load from files using `_FILE` suffix
//! - **Type-safe**: Compile-time type checking
//! - **Default values**: Support for `Default` trait and explicit values
//! - **Plain values**: Environment variables use plain text format (no JSON by default)
//! - **Custom deserializers**: Support for JSON, TOML, or custom parsing functions
//!
//! # Value Parsing
//!
//! **Default (using `FromStr`)**:
//! - Strings: `DATABASE_URL=postgres://localhost/db`
//! - Numbers: `MAX_CONNECTIONS=42`
//! - Booleans: `DEBUG=true`
//!
//! **Custom deserializers** - specify with `#[env(deserializer = "function")]`:
//! - JSON: `#[env(deserializer = "serde_json::from_str")]`
//! - TOML: `#[env(deserializer = "toml::from_str")]`
//! - Custom: Define your own deserializer function
//!
//! # Example
//!
//! ```rust
//! use envconf::EnvConf;
//!
//! #[derive(Debug, EnvConf)]
//! struct Config {
//!     pub database_url: String,
//!
//!     #[env(default = "127.0.0.1:8080".to_string())]
//!     pub server_addr: String,
//!
//!     #[env(default = 10)]
//!     pub max_connections: u32,
//!
//!     #[env(default)]
//!     pub debug_mode: bool,
//! }
//!
//! # fn main() -> anyhow::Result<()> {
//! #     std::env::set_var("DATABASE_URL", "postgres://localhost/db");
//! #     let config = Config::from_env()?;
//! #     assert_eq!(config.database_url, "postgres://localhost/db");
//! #     Ok(())
//! # }
//! ```
//!
//! # Attributes
//!
//! ## `#[env(from_file)]`
//!
//! Load from `{VAR_NAME}_FILE` in addition to the environment variable.
//! Useful for handling file-based configuration like Kubernetes Secrets.
//!
//! ```rust
//! # use envconf::EnvConf;
//! #[derive(EnvConf)]
//! pub struct Config {
//!     // Load from DATABASE_URL or DATABASE_URL_FILE
//!     #[env(from_file)]
//!     pub database_url: String,
//! }
//! ```
//!
//! ## `#[env(default = "value")]`
//!
//! Specify a default value when the environment variable is not set.
//!
//! ```rust
//! # use envconf::EnvConf;
//! #[derive(EnvConf)]
//! struct Config {
//!     #[env(default = "127.0.0.1:8080".to_string())]
//!     pub server_addr: String,
//!
//!     #[env(default = 10)]
//!     pub max_connections: u32,
//!
//!     #[env(default = false)]
//!     pub enable_tls: bool,
//! }
//! # fn main() -> anyhow::Result<()> {
//! #     let config = Config::from_env()?;
//! #     assert_eq!(config.max_connections, 10);
//! #     Ok(())
//! # }
//! ```
//!
//! ## `#[env(name = "CUSTOM_NAME")]`
//!
//! Specify an environment variable name different from the field name.
//!
//! ```rust
//! # use envconf::EnvConf;
//! #[derive(EnvConf)]
//! pub struct Config {
//!     // Load from REDIS_URL environment variable
//!     #[env(name = "REDIS_URL")]
//!     pub redis_connection_string: String,
//! }
//! ```

#[doc(hidden)]
pub mod de;

mod error;

pub use envconf_derive::EnvConf;
pub use error::EnvError;

// Re-export for macro-generated code
#[doc(hidden)]
pub use anyhow;
