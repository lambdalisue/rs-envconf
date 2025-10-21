//! Example demonstrating prefix attribute

use serviceconf::ServiceConf;

#[derive(Debug, ServiceConf)]
#[conf(prefix = "MYAPP_")]
struct Config {
    // Environment variables will be prefixed: MYAPP_DATABASE_URL, MYAPP_API_KEY, etc.
    pub database_url: String,
    pub api_key: String,

    #[conf(default = 8080)]
    pub port: u16,

    #[conf(default)]
    pub debug: bool,
}

fn main() -> anyhow::Result<()> {
    // Set environment variables with prefix
    std::env::set_var("MYAPP_DATABASE_URL", "postgres://localhost/db");
    std::env::set_var("MYAPP_API_KEY", "secret-key-123");
    std::env::set_var("MYAPP_PORT", "3000");

    let config = Config::from_env()?;

    println!("Configuration with prefix 'MYAPP_':");
    println!("  Database URL: {}", config.database_url);
    println!("  API Key: {}", config.api_key);
    println!("  Port: {}", config.port);
    println!("  Debug: {}", config.debug);

    Ok(())
}
