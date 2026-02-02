//! Config directory example
//!
//! Demonstrates struct-level config_dir attributes for file-based secrets

use serviceconf::ServiceConf;
use std::fs;
use tempfile::TempDir;

#[derive(Debug, ServiceConf)]
#[conf(config_dir_env = "SECRETS_DIR", config_dir = "/secrets")]
struct Config {
    // File-based configuration with config_dir fallback
    #[conf(from_file)]
    pub api_key: String,

    #[conf(from_file)]
    pub database_password: String,

    // Regular environment variable (not affected by config_dir)
    pub database_host: String,

    // Optional field with config_dir support
    #[conf(from_file)]
    pub optional_token: Option<String>,

    // Field with default and config_dir support
    #[conf(from_file, default = "-o -p".to_string())]
    pub backup_options: String,
}

fn main() -> anyhow::Result<()> {
    // Create a temporary directory to simulate /secrets
    let temp_dir = TempDir::new()?;
    let secrets_dir = temp_dir.path();

    // Create secret files in the directory
    fs::write(secrets_dir.join("API_KEY"), "secret_api_key_from_dir")?;
    fs::write(secrets_dir.join("DATABASE_PASSWORD"), "secret_db_pass_from_dir")?;
    fs::write(secrets_dir.join("OPTIONAL_TOKEN"), "optional_token_from_dir")?;
    // for BACKUP_OPTIONS we'll use default value

    // Set the config directory via environment variable
    std::env::set_var("SECRETS_DIR", secrets_dir);
    std::env::set_var("DATABASE_HOST", "localhost");

    println!("=== Config Directory Example ===");
    println!("Config directory: {}", secrets_dir.display());
    println!();

    // Load configuration
    let config = Config::from_env()?;

    println!("Configuration loaded:");
    println!("  API Key: {}", config.api_key);
    println!("  Database Password: {}", config.database_password);
    println!("  Database Host: {}", config.database_host);
    println!("  Optional Token: {:?}", config.optional_token);
    println!("  Backup options: {}", config.backup_options);
    println!();


    // Remove SECRETS_DIR to test fallback to hardcoded path
    std::env::remove_var("SECRETS_DIR");
    
    // This would normally fail since /secrets doesn't exist,
    // but we'll catch the error to demonstrate the behavior
    match Config::from_env() {
        Ok(config) => println!("Loaded with hardcoded path: {:?}", config),
        Err(e) => println!("Expected error with hardcoded path: {}", e),
    }

    Ok(())
}

