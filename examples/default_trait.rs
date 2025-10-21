//! Example demonstrating #[conf(default)] using Default trait

use serviceconf::ServiceConf;

#[derive(Debug, ServiceConf)]
struct Config {
    // Uses Default::default() if not set
    #[conf(default)]
    pub host: String, // "" (empty string)

    #[conf(default)]
    pub port: u16, // 0

    #[conf(default)]
    pub enabled: bool, // false

    // Explicit default values for comparison
    #[conf(default = "localhost".to_string())]
    pub server: String,

    #[conf(default = 8080)]
    pub api_port: u16,

    #[conf(default = true)]
    pub verbose: bool,
}

fn main() -> anyhow::Result<()> {
    // Don't set any environment variables
    std::env::remove_var("HOST");
    std::env::remove_var("PORT");
    std::env::remove_var("ENABLED");
    std::env::remove_var("SERVER");
    std::env::remove_var("API_PORT");
    std::env::remove_var("VERBOSE");

    let config = Config::from_env()?;

    println!("Configuration with defaults:");
    println!("  Host (Default trait): '{}'", config.host); // ""
    println!("  Port (Default trait): {}", config.port); // 0
    println!("  Enabled (Default trait): {}", config.enabled); // false
    println!("  Server (explicit): '{}'", config.server); // "localhost"
    println!("  API Port (explicit): {}", config.api_port); // 8080
    println!("  Verbose (explicit): {}", config.verbose); // true

    Ok(())
}
