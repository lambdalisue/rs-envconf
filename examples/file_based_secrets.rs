//! File-based configuration example

use serviceconf::ServiceConf;
use std::io::Write;
use tempfile::NamedTempFile;

#[derive(Debug, ServiceConf)]
struct Config {
    // File-based configuration: load from API_KEY or API_KEY_FILE
    #[conf(from_file)]
    pub api_key: String,

    // Database password can also be loaded from a file
    #[conf(from_file)]
    pub database_password: String,

    // Regular environment variable
    pub database_host: String,
}

fn main() -> anyhow::Result<()> {
    // Save API key to file
    let mut api_key_file = NamedTempFile::new()?;
    writeln!(api_key_file, "super_secret_api_key_12345")?;

    // Save database password to file
    let mut db_password_file = NamedTempFile::new()?;
    writeln!(db_password_file, "db_password_67890")?;

    // Set environment variables (with _FILE suffix)
    std::env::set_var("API_KEY_FILE", api_key_file.path());
    std::env::set_var("DATABASE_PASSWORD_FILE", db_password_file.path());
    std::env::set_var("DATABASE_HOST", "localhost");

    // Load configuration
    let config = Config::from_env()?;

    println!("Configuration loaded from files:");
    println!("  API Key: {}", config.api_key);
    println!("  Database Password: {}", config.database_password);
    println!("  Database Host: {}", config.database_host);

    Ok(())
}
