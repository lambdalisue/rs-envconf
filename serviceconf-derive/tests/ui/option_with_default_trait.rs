// This test verifies that using default (trait) with Option<T> produces a clear error
// Option<T> fields automatically default to None, so Default::default() is redundant

use serviceconf::ServiceConf;

#[derive(ServiceConf)]
struct Config {
    /// This should produce a clear error
    #[conf(default)]
    pub optional_field: Option<String>,
}

fn main() {}
