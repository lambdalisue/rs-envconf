//! Example demonstrating Option<T> for optional fields

use serviceconf::ServiceConf;

#[derive(Debug, ServiceConf)]
struct Config {
    // Required field
    pub app_name: String,

    // Optional fields - None if not set
    pub api_key: Option<String>,
    pub port: Option<u16>,
    pub debug: Option<bool>,

    // Optional with from_file
    #[conf(from_file)]
    pub database_password: Option<String>,
}

fn main() -> anyhow::Result<()> {
    // Set only some environment variables
    std::env::set_var("APP_NAME", "my-application");
    std::env::set_var("PORT", "8080");
    // API_KEY, DEBUG, DATABASE_PASSWORD not set

    let config = Config::from_env()?;

    println!("Configuration:");
    println!("  App Name: {}", config.app_name);
    println!("  API Key: {:?}", config.api_key); // None
    println!("  Port: {:?}", config.port); // Some(8080)
    println!("  Debug: {:?}", config.debug); // None
    println!("  Database Password: {:?}", config.database_password); // None

    Ok(())
}
