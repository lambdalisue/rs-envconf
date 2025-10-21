// This test verifies that using default with Option<T> produces a clear error
// Option<T> fields automatically default to None, so explicit default is redundant

use serviceconf::ServiceConf;

#[derive(ServiceConf)]
struct Config {
    /// This should produce a clear error
    #[conf(default = "fallback".to_string())]
    pub optional_field: Option<String>,
}

fn main() {}
