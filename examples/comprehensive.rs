//! Comprehensive example showing multiple features combined

use serviceconf::ServiceConf;
use std::collections::HashMap;

#[derive(Debug, ServiceConf)]
#[conf(prefix = "APP_")]
struct Config {
    // Required field
    pub name: String, // APP_NAME

    // Optional field
    pub version: Option<String>, // APP_VERSION

    // Default value
    #[conf(default = 8080)]
    pub port: u16, // APP_PORT

    // Default trait
    #[conf(default)]
    pub debug: bool, // APP_DEBUG

    // Custom name
    #[conf(name = "DATABASE_CONNECTION_STRING")]
    pub database_url: String, // APP_DATABASE_CONNECTION_STRING

    // File-based secret
    #[conf(from_file)]
    pub api_key: String, // APP_API_KEY or APP_API_KEY_FILE

    // Optional file-based secret
    #[conf(from_file)]
    pub oauth_token: Option<String>, // APP_OAUTH_TOKEN or APP_OAUTH_TOKEN_FILE

    // Custom deserializer
    #[conf(deserializer = "serde_json::from_str")]
    pub tags: Vec<String>, // APP_TAGS (JSON)

    // Optional with custom deserializer
    #[conf(deserializer = "serde_json::from_str")]
    pub metadata: Option<HashMap<String, String>>, // APP_METADATA (JSON)
}

fn main() -> anyhow::Result<()> {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Required fields
    std::env::set_var("APP_NAME", "my-application");
    std::env::set_var("APP_DATABASE_CONNECTION_STRING", "postgres://localhost/db");
    std::env::set_var("APP_TAGS", r#"["production","api"]"#);

    // Optional field - set this one
    std::env::set_var("APP_VERSION", "1.0.0");

    // Port will use default (8080)
    // Debug will use Default::default() (false)

    // File-based secret
    let mut api_key_file = NamedTempFile::new()?;
    writeln!(api_key_file, "super-secret-key")?;
    std::env::set_var("APP_API_KEY_FILE", api_key_file.path());

    // OAuth token not set - will be None

    // Metadata not set - will be None

    let config = Config::from_env()?;

    println!("Comprehensive Configuration:");
    println!("  Name: {}", config.name);
    println!("  Version: {:?}", config.version);
    println!("  Port: {}", config.port);
    println!("  Debug: {}", config.debug);
    println!("  Database URL: {}", config.database_url);
    println!("  API Key: {}", config.api_key);
    println!("  OAuth Token: {:?}", config.oauth_token);
    println!("  Tags: {:?}", config.tags);
    println!("  Metadata: {:?}", config.metadata);

    Ok(())
}
