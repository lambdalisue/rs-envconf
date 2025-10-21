// This test verifies that the error message for using deserializer with default
// is clear and properly formatted (not a confusing "expected identifier" error)

use serviceconf::ServiceConf;
use std::time::Duration;

fn parse_duration_secs(s: &str) -> Result<Duration, String> {
    s.parse::<u64>()
        .map(Duration::from_secs)
        .map_err(|e| format!("Failed to parse: {}", e))
}

#[derive(ServiceConf)]
struct Config {
    /// English comment with deserializer + default should give clear error
    #[conf(deserializer = "parse_duration_secs", default = Duration::from_secs(60))]
    pub timeout: Duration,
}

fn main() {}
