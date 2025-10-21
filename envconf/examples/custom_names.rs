//! Custom environment variable names example

use envconf::EnvConf;

#[derive(Debug, EnvConf)]
struct Config {
    // Load from POSTGRES_CONNECTION_STRING environment variable
    #[env(name = "POSTGRES_CONNECTION_STRING")]
    pub database_url: String,

    // Load from REDIS_ENDPOINT environment variable
    #[env(name = "REDIS_ENDPOINT")]
    pub cache_url: String,

    // Load from SERVER_PORT environment variable (with default value)
    #[env(default = 8080)]
    pub server_port: u16,
}

fn main() -> anyhow::Result<()> {
    // Set environment variables
    std::env::set_var(
        "POSTGRES_CONNECTION_STRING",
        "postgres://user:pass@localhost/db",
    );
    std::env::set_var("REDIS_ENDPOINT", "redis://localhost:6379");

    // Load configuration
    let config = Config::from_env()?;

    println!("Configuration with custom env names:");
    println!("  Database URL: {}", config.database_url);
    println!("  Cache URL: {}", config.cache_url);
    println!("  Server Port: {}", config.server_port);

    Ok(())
}
