//! Example demonstrating #[env(default)] using Default trait

use envconf::EnvConf;

#[derive(Debug, EnvConf)]
struct Config {
    // Uses Default::default() if not set
    #[env(default)]
    pub host: String, // "" (empty string)

    #[env(default)]
    pub port: u16, // 0

    #[env(default)]
    pub enabled: bool, // false

    // Explicit default values for comparison
    #[env(default = "localhost".to_string())]
    pub server: String,

    #[env(default = 8080)]
    pub api_port: u16,

    #[env(default = true)]
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
