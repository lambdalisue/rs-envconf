//! Basic usage example

use serviceconf::ServiceConf;

#[derive(Debug, ServiceConf)]
struct Config {
    // Required field: loaded from DATABASE_URL environment variable
    pub database_url: String,

    // With default value
    #[conf(default = "127.0.0.1:8080".to_string())]
    pub server_addr: String,

    // Numeric type
    #[conf(default = 10)]
    pub max_connections: u32,

    // Boolean type
    #[conf(default = false)]
    pub debug_mode: bool,
}

fn main() -> anyhow::Result<()> {
    // Set environment variables for demonstration
    std::env::set_var("DATABASE_URL", "postgres://localhost/mydb");
    std::env::set_var("SERVER_ADDR", "0.0.0.0:3000");

    // Load configuration
    let config = Config::from_env()?;

    println!("Configuration loaded:");
    println!("  Database URL: {}", config.database_url);
    println!("  Server Address: {}", config.server_addr);
    println!("  Max Connections: {}", config.max_connections);
    println!("  Debug Mode: {}", config.debug_mode);

    Ok(())
}
