//! Example demonstrating custom deserializer functions

use envconf::EnvConf;

// Custom deserializer for comma-separated strings
fn comma_separated(s: &str) -> Result<Vec<String>, String> {
    Ok(s.split(',').map(|s| s.trim().to_string()).collect())
}

#[derive(Debug, EnvConf)]
struct Config {
    // Default: uses FromStr
    pub app_name: String,
    pub port: u16,

    // Uses serde_json::from_str (JSON format)
    #[env(deserializer = "serde_json::from_str")]
    pub json_tags: Vec<String>,

    // Uses custom function (comma-separated)
    #[env(deserializer = "comma_separated")]
    pub comma_tags: Vec<String>,
    // You can also use any other deserializer like toml::from_str
    // #[env(deserializer = "toml::from_str")]
    // pub toml_config: MyTomlConfig,
}

fn main() -> anyhow::Result<()> {
    // Simple values (FromStr)
    std::env::set_var("APP_NAME", "my-app");
    std::env::set_var("PORT", "8080");

    // JSON format (serde_json)
    std::env::set_var("JSON_TAGS", r#"["prod","api","v2"]"#);

    // Comma-separated (custom function)
    std::env::set_var("COMMA_TAGS", "tag1, tag2, tag3");

    // Load configuration
    let config = Config::from_env()?;

    println!("Configuration loaded:");
    println!("  App Name: {}", config.app_name);
    println!("  Port: {}", config.port);
    println!("  JSON Tags: {:?}", config.json_tags);
    println!("  Comma Tags: {:?}", config.comma_tags);

    Ok(())
}
