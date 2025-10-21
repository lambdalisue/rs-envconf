// This test verifies that using deserializer with default (trait) produces a clear error

use serviceconf::ServiceConf;

fn parse_list(s: &str) -> Result<Vec<String>, String> {
    Ok(s.split(',').map(|s| s.trim().to_string()).collect())
}

#[derive(ServiceConf)]
struct Config {
    /// This should produce a clear error
    #[conf(deserializer = "parse_list", default)]
    pub items: Vec<String>,
}

fn main() {}
