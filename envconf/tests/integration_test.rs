//! Integration tests

use envconf::EnvConf;
use serial_test::serial;
use std::env;

#[derive(Debug, EnvConf)]
struct BasicConfig {
    pub database_url: String,
    pub api_key: String,
}

#[derive(Debug, EnvConf)]
struct ConfigWithDefaults {
    #[env(default = "127.0.0.1:8080".to_string())]
    pub server_addr: String,

    #[env(default = 10)]
    pub max_connections: u32,

    #[env(default = false)]
    pub debug_mode: bool,
}

#[derive(Debug, EnvConf)]
struct ConfigWithCustomNames {
    #[env(name = "DB_CONNECTION_STRING")]
    pub database_url: String,

    #[env(name = "REDIS_URL")]
    pub cache_url: String,
}

#[derive(Debug, EnvConf)]
struct ConfigWithFileSupport {
    #[env(from_file)]
    pub secret_key: String,

    pub normal_var: String,
}

#[test]
#[serial]
fn test_basic_config() {
    env::set_var("DATABASE_URL", "postgres://localhost/test");
    env::set_var("API_KEY", "test_api_key");

    let config = BasicConfig::from_env().unwrap();
    assert_eq!(config.database_url, "postgres://localhost/test");
    assert_eq!(config.api_key, "test_api_key");

    env::remove_var("DATABASE_URL");
    env::remove_var("API_KEY");
}

#[test]
#[serial]
fn test_missing_required_field() {
    env::remove_var("DATABASE_URL");
    env::remove_var("API_KEY");

    let result = BasicConfig::from_env();
    assert!(result.is_err());
}

#[test]
#[serial]
fn test_config_with_defaults() {
    env::remove_var("SERVER_ADDR");
    env::remove_var("MAX_CONNECTIONS");
    env::remove_var("DEBUG_MODE");

    let config = ConfigWithDefaults::from_env().unwrap();
    assert_eq!(config.server_addr, "127.0.0.1:8080");
    assert_eq!(config.max_connections, 10);
    assert!(!config.debug_mode);
}

#[test]
#[serial]
fn test_config_override_defaults() {
    env::set_var("SERVER_ADDR", "0.0.0.0:9090");
    env::set_var("MAX_CONNECTIONS", "20");
    env::set_var("DEBUG_MODE", "true");

    let config = ConfigWithDefaults::from_env().unwrap();
    assert_eq!(config.server_addr, "0.0.0.0:9090");
    assert_eq!(config.max_connections, 20);
    assert!(config.debug_mode);

    env::remove_var("SERVER_ADDR");
    env::remove_var("MAX_CONNECTIONS");
    env::remove_var("DEBUG_MODE");
}

#[test]
#[serial]
fn test_custom_env_names() {
    env::set_var("DB_CONNECTION_STRING", "postgres://localhost/db");
    env::set_var("REDIS_URL", "redis://localhost");

    let config = ConfigWithCustomNames::from_env().unwrap();
    assert_eq!(config.database_url, "postgres://localhost/db");
    assert_eq!(config.cache_url, "redis://localhost");

    env::remove_var("DB_CONNECTION_STRING");
    env::remove_var("REDIS_URL");
}

#[test]
#[serial]
fn test_file_based_config() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "super_secret_key").unwrap();

    env::set_var("SECRET_KEY_FILE", temp_file.path());
    env::set_var("NORMAL_VAR", "normal_value");
    env::remove_var("SECRET_KEY");

    let config = ConfigWithFileSupport::from_env().unwrap();
    assert_eq!(config.secret_key, "super_secret_key");
    assert_eq!(config.normal_var, "normal_value");

    env::remove_var("SECRET_KEY_FILE");
    env::remove_var("NORMAL_VAR");
}

#[test]
#[serial]
fn test_direct_var_preferred_over_file() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "file_value").unwrap();

    env::set_var("SECRET_KEY", "direct_value");
    env::set_var("SECRET_KEY_FILE", temp_file.path());
    env::set_var("NORMAL_VAR", "normal");

    let config = ConfigWithFileSupport::from_env().unwrap();
    assert_eq!(config.secret_key, "direct_value");

    env::remove_var("SECRET_KEY");
    env::remove_var("SECRET_KEY_FILE");
    env::remove_var("NORMAL_VAR");
}

#[test]
#[serial]
fn test_parse_error() {
    env::set_var("MAX_CONNECTIONS", "not_a_number");

    let result = ConfigWithDefaults::from_env();
    assert!(result.is_err());

    env::remove_var("MAX_CONNECTIONS");
}

#[derive(Debug, EnvConf)]
struct ConfigWithJsonFields {
    // Simple field using FromStr
    pub simple_value: String,

    // Complex field using JSON deserialization
    #[env(deserializer = "serde_json::from_str")]
    pub tags: Vec<String>,
}

#[test]
#[serial]
fn test_json_deserialization() {
    env::set_var("SIMPLE_VALUE", "hello");
    env::set_var("TAGS", r#"["tag1","tag2","tag3"]"#);

    let config = ConfigWithJsonFields::from_env().unwrap();
    assert_eq!(config.simple_value, "hello");
    assert_eq!(config.tags, vec!["tag1", "tag2", "tag3"]);

    env::remove_var("SIMPLE_VALUE");
    env::remove_var("TAGS");
}

fn comma_separated_deserializer(s: &str) -> Result<Vec<String>, String> {
    Ok(s.split(',').map(|s| s.trim().to_string()).collect())
}

#[derive(Debug, EnvConf)]
struct ConfigWithCustomDeserializer {
    pub simple: String,

    #[env(deserializer = "comma_separated_deserializer")]
    pub comma_list: Vec<String>,
}

#[test]
#[serial]
fn test_custom_deserializer_function() {
    env::set_var("SIMPLE", "value");
    env::set_var("COMMA_LIST", "a, b, c");

    let config = ConfigWithCustomDeserializer::from_env().unwrap();
    assert_eq!(config.simple, "value");
    assert_eq!(config.comma_list, vec!["a", "b", "c"]);

    env::remove_var("SIMPLE");
    env::remove_var("COMMA_LIST");
}

#[derive(Debug, EnvConf)]
struct ConfigWithDefaultTrait {
    // Uses Default::default() when env var is missing
    #[env(default)]
    pub optional_string: String,

    #[env(default)]
    pub optional_number: u32,

    #[env(default)]
    pub optional_bool: bool,
}

#[test]
#[serial]
fn test_default_trait() {
    env::remove_var("OPTIONAL_STRING");
    env::remove_var("OPTIONAL_NUMBER");
    env::remove_var("OPTIONAL_BOOL");

    let config = ConfigWithDefaultTrait::from_env().unwrap();
    assert_eq!(config.optional_string, ""); // String::default()
    assert_eq!(config.optional_number, 0); // u32::default()
    assert!(!config.optional_bool); // bool::default()
}

#[derive(Debug, EnvConf)]
struct ConfigWithDefaultAndFile {
    // Combination of default and from_file
    #[env(from_file)]
    #[env(default = "default_secret".to_string())]
    pub secret_with_default: String,

    #[env(from_file)]
    #[env(default)]
    pub optional_secret: String,
}

#[test]
#[serial]
fn test_default_with_from_file() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Test 1: File exists
    let mut secret_file = NamedTempFile::new().unwrap();
    writeln!(secret_file, "file_secret").unwrap();

    env::set_var("SECRET_WITH_DEFAULT_FILE", secret_file.path());
    env::remove_var("SECRET_WITH_DEFAULT");
    env::remove_var("OPTIONAL_SECRET");
    env::remove_var("OPTIONAL_SECRET_FILE");

    let config = ConfigWithDefaultAndFile::from_env().unwrap();
    assert_eq!(config.secret_with_default, "file_secret");
    assert_eq!(config.optional_secret, ""); // Default::default()

    // Test 2: No file, use default
    env::remove_var("SECRET_WITH_DEFAULT_FILE");
    let config = ConfigWithDefaultAndFile::from_env().unwrap();
    assert_eq!(config.secret_with_default, "default_secret");

    env::remove_var("SECRET_WITH_DEFAULT");
}

#[test]
#[serial]
fn test_env_var_overrides_file() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut secret_file = NamedTempFile::new().unwrap();
    writeln!(secret_file, "file_value").unwrap();

    env::set_var("SECRET_WITH_DEFAULT", "env_value");
    env::set_var("SECRET_WITH_DEFAULT_FILE", secret_file.path());

    let config = ConfigWithDefaultAndFile::from_env().unwrap();
    // Direct env var should take precedence
    assert_eq!(config.secret_with_default, "env_value");

    env::remove_var("SECRET_WITH_DEFAULT");
    env::remove_var("SECRET_WITH_DEFAULT_FILE");
}

#[derive(Debug, EnvConf)]
#[env(prefix = "APP_")]
struct ConfigWithPrefix {
    pub database_url: String,
    pub api_key: String,

    #[env(default = 8080)]
    pub port: u16,
}

#[test]
#[serial]
fn test_prefix() {
    // With prefix "APP_", field names become APP_DATABASE_URL, APP_API_KEY, APP_PORT
    env::set_var("APP_DATABASE_URL", "postgres://localhost/db");
    env::set_var("APP_API_KEY", "secret123");
    env::remove_var("APP_PORT"); // Use default

    let config = ConfigWithPrefix::from_env().unwrap();
    assert_eq!(config.database_url, "postgres://localhost/db");
    assert_eq!(config.api_key, "secret123");
    assert_eq!(config.port, 8080);

    env::remove_var("APP_DATABASE_URL");
    env::remove_var("APP_API_KEY");
}

#[derive(Debug, EnvConf)]
struct ConfigWithOption {
    pub required: String,
    pub optional: Option<String>,
    pub optional_number: Option<u32>,
}

#[test]
#[serial]
fn test_option_type_some() {
    env::set_var("REQUIRED", "required_value");
    env::set_var("OPTIONAL", "optional_value");
    env::set_var("OPTIONAL_NUMBER", "42");

    let config = ConfigWithOption::from_env().unwrap();
    assert_eq!(config.required, "required_value");
    assert_eq!(config.optional, Some("optional_value".to_string()));
    assert_eq!(config.optional_number, Some(42));

    env::remove_var("REQUIRED");
    env::remove_var("OPTIONAL");
    env::remove_var("OPTIONAL_NUMBER");
}

#[test]
#[serial]
fn test_option_type_none() {
    env::set_var("REQUIRED", "required_value");
    env::remove_var("OPTIONAL");
    env::remove_var("OPTIONAL_NUMBER");

    let config = ConfigWithOption::from_env().unwrap();
    assert_eq!(config.required, "required_value");
    assert_eq!(config.optional, None);
    assert_eq!(config.optional_number, None);

    env::remove_var("REQUIRED");
}

#[derive(Debug, EnvConf)]
struct ConfigWithOptionAndFile {
    #[env(from_file)]
    pub optional_secret: Option<String>,
}

#[test]
#[serial]
fn test_option_with_from_file() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Test 1: File exists
    let mut secret_file = NamedTempFile::new().unwrap();
    writeln!(secret_file, "file_secret").unwrap();

    env::set_var("OPTIONAL_SECRET_FILE", secret_file.path());
    env::remove_var("OPTIONAL_SECRET");

    let config = ConfigWithOptionAndFile::from_env().unwrap();
    assert_eq!(config.optional_secret, Some("file_secret".to_string()));

    // Test 2: No file, no env var -> None
    env::remove_var("OPTIONAL_SECRET");
    env::remove_var("OPTIONAL_SECRET_FILE");

    let config = ConfigWithOptionAndFile::from_env().unwrap();
    assert_eq!(config.optional_secret, None);
}

fn json_deserializer(s: &str) -> Result<Vec<String>, serde_json::Error> {
    serde_json::from_str(s)
}

#[derive(Debug, EnvConf)]
struct ConfigWithOptionAndDeserializer {
    #[env(deserializer = "json_deserializer")]
    pub optional_tags: Option<Vec<String>>,
}

#[test]
#[serial]
fn test_option_with_deserializer() {
    // Test 1: Value exists
    env::set_var("OPTIONAL_TAGS", r#"["tag1","tag2"]"#);

    let config = ConfigWithOptionAndDeserializer::from_env().unwrap();
    assert_eq!(
        config.optional_tags,
        Some(vec!["tag1".to_string(), "tag2".to_string()])
    );

    // Test 2: Value doesn't exist -> None
    env::remove_var("OPTIONAL_TAGS");

    let config = ConfigWithOptionAndDeserializer::from_env().unwrap();
    assert_eq!(config.optional_tags, None);
}

#[derive(Debug, EnvConf)]
#[env(prefix = "TEST_")]
struct ConfigWithPrefixAndCustomName {
    #[env(name = "DB")]
    pub database_url: String, // Reads from TEST_DB (not TEST_DATABASE_URL)
}

#[test]
#[serial]
fn test_prefix_with_custom_name() {
    env::set_var("TEST_DB", "postgres://localhost/db");

    let config = ConfigWithPrefixAndCustomName::from_env().unwrap();
    assert_eq!(config.database_url, "postgres://localhost/db");

    env::remove_var("TEST_DB");
}

#[test]
#[serial]
fn test_file_read_error() {
    env::set_var("SECRET_FILE", "/nonexistent/path/to/file");
    env::remove_var("SECRET");

    #[derive(EnvConf)]
    #[allow(dead_code)]
    struct TempConfig {
        #[env(from_file)]
        secret: String,
    }

    let result = TempConfig::from_env();
    assert!(result.is_err());

    // Verify it's a FileRead error
    if let Err(e) = result {
        let err_str = e.to_string();
        assert!(err_str.contains("Failed to read file"));
    }

    env::remove_var("SECRET_FILE");
}
