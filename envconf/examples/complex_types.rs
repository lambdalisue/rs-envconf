//! Example demonstrating complex types with explicit JSON deserialization

use envconf::EnvConf;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
}

impl DatabaseConfig {
    fn connection_string(&self) -> String {
        format!("{}:{} (user: {})", self.host, self.port, self.username)
    }
}

#[derive(Debug, EnvConf)]
struct Config {
    // Simple string using FromStr (no JSON required)
    pub app_name: String,

    // Number using FromStr
    pub max_connections: u32,

    // Complex type using explicit JSON deserialization
    #[env(deserializer = "serde_json::from_str")]
    pub tags: Vec<String>,

    // HashMap using explicit JSON deserialization
    #[env(deserializer = "serde_json::from_str")]
    pub environment_vars: HashMap<String, String>,

    // Nested struct using explicit JSON deserialization
    #[env(deserializer = "serde_json::from_str")]
    pub database: DatabaseConfig,
}

fn main() -> anyhow::Result<()> {
    // Simple values - plain text (using FromStr)
    std::env::set_var("APP_NAME", "my-application");
    std::env::set_var("MAX_CONNECTIONS", "100");

    // Complex values - JSON format (using serde)
    std::env::set_var("TAGS", r#"["production","api","v2"]"#);
    std::env::set_var(
        "ENVIRONMENT_VARS",
        r#"{"LOG_LEVEL":"debug","TIMEOUT":"30"}"#,
    );
    std::env::set_var(
        "DATABASE",
        r#"{"host":"localhost","port":5432,"username":"admin"}"#,
    );

    // Load configuration
    let config = Config::from_env()?;

    println!("Configuration loaded:");
    println!("  App Name: {}", config.app_name);
    println!("  Max Connections: {}", config.max_connections);
    println!("  Tags: {:?}", config.tags);
    println!("  Environment Variables: {:?}", config.environment_vars);
    println!("  Database: {}", config.database.connection_string());

    Ok(())
}
