//! Deserialization functions for environment variables.
//!
//! This module provides internal functions used by the `ServiceConf` derive macro
//! to load and parse configuration values from environment variables.
//!
//! All functions in this module support the `{VAR}_FILE` pattern for file-based secrets,
//! which is the primary feature distinguishing this library from other environment
//! configuration solutions.

use crate::error::ServiceConfError;
use std::env;
use std::fs;
use std::str::FromStr;

/// Load a required value using `FromStr`
///
/// Used by the derive macro for fields without default values.
#[doc(hidden)]
pub fn deserialize_required<T>(env_name: &str, from_file: bool, config_dir: Option<&str>) -> Result<T, ServiceConfError>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let value = get_env_value(env_name, from_file, config_dir)?;
    value
        .parse::<T>()
        .map_err(|e| ServiceConfError::parse_error::<T>(env_name, e))
}

/// Load a value with a default using `FromStr`
///
/// Used by the derive macro for fields with default values.
#[doc(hidden)]
pub fn deserialize_with_default<T>(
    env_name: &str,
    from_file: bool,
    config_dir: Option<&str>,
    default: T,
) -> Result<T, ServiceConfError>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match get_env_value(env_name, from_file, config_dir) {
        Ok(value) => value
            .parse::<T>()
            .map_err(|e| ServiceConfError::parse_error::<T>(env_name, e)),
        Err(ServiceConfError::Missing { .. }) => Ok(default),
        Err(e) => Err(e),
    }
}

/// Load an optional value using `FromStr`
///
/// Returns `None` if environment variable is not set, `Some(T)` if it is.
/// Used by the derive macro for `Option<T>` fields.
#[doc(hidden)]
pub fn deserialize_optional<T>(
    env_name: &str,
    from_file: bool,
    config_dir: Option<&str>,
) -> Result<Option<T>, ServiceConfError>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match get_env_value(env_name, from_file, config_dir) {
        Ok(value) => {
            let parsed = value
                .parse::<T>()
                .map_err(|e| ServiceConfError::parse_error::<T>(env_name, e))?;
            Ok(Some(parsed))
        }
        Err(ServiceConfError::Missing { .. }) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Get environment variable value with optional file-based fallback
///
/// Priority order:
/// 1. Direct environment variable (`env_name`)
/// 2. File from environment variable (`{env_name}_FILE`) if `from_file` is true
/// 3. File from config directory (`{config_dir}/{env_name}`) if `config_dir` is provided and `from_file` is true
/// 4. Error if none are found
///
/// Used by macro-generated code.
#[doc(hidden)]
pub fn get_env_value(env_name: &str, from_file: bool, config_dir: Option<&str>) -> Result<String, ServiceConfError> {
    if let Ok(value) = env::var(env_name) {
        return Ok(value);
    }

    if from_file {
        let file_var_name = format!("{}_FILE", env_name);
        if let Ok(file_path) = env::var(&file_var_name) {
            return fs::read_to_string(&file_path)
                .map(|s| s.trim().to_string())
                .map_err(|e| ServiceConfError::FileRead {
                    name: file_var_name,
                    path: file_path,
                    source: e,
                });
        }

        // Try config directory fallback
        if let Some(dir) = config_dir {
            let file_path = format!("{}/{}", dir, env_name);
            if let Ok(content) = fs::read_to_string(&file_path) {
                return Ok(content.trim().to_string());
            }
        }
    }

    Err(ServiceConfError::missing(env_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_deserialize_required_success() {
        env::set_var("TEST_VAR", "42");
        let result: Result<i32, _> = deserialize_required("TEST_VAR", false, None);
        assert_eq!(result.unwrap(), 42);
        env::remove_var("TEST_VAR");
    }

    #[test]
    #[serial]
    fn test_deserialize_required_missing() {
        env::remove_var("MISSING_VAR");
        let result: Result<String, _> = deserialize_required("MISSING_VAR", false, None);
        assert!(matches!(result, Err(ServiceConfError::Missing { .. })));
    }

    #[test]
    #[serial]
    fn test_deserialize_with_default_env_set() {
        env::set_var("TEST_DEFAULT", "100");
        let result: u32 = deserialize_with_default("TEST_DEFAULT", false, None, 50).unwrap();
        assert_eq!(result, 100);
        env::remove_var("TEST_DEFAULT");
    }

    #[test]
    #[serial]
    fn test_deserialize_with_default_use_default() {
        env::remove_var("TEST_DEFAULT_MISSING");
        let result: u32 = deserialize_with_default("TEST_DEFAULT_MISSING", false, None, 50).unwrap();
        assert_eq!(result, 50);
    }

    #[test]
    #[serial]
    fn test_get_env_value_from_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "secret_value").unwrap();

        env::set_var("TEST_FILE_VAR_FILE", temp_file.path());
        env::remove_var("TEST_FILE_VAR");

        let result = get_env_value("TEST_FILE_VAR", true, None).unwrap();
        assert_eq!(result, "secret_value");

        env::remove_var("TEST_FILE_VAR_FILE");
    }

    #[test]
    #[serial]
    fn test_get_env_value_prefers_direct() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "file_value").unwrap();

        env::set_var("TEST_PREFER", "direct_value");
        env::set_var("TEST_PREFER_FILE", temp_file.path());

        let result = get_env_value("TEST_PREFER", true, None).unwrap();
        assert_eq!(result, "direct_value");

        env::remove_var("TEST_PREFER");
        env::remove_var("TEST_PREFER_FILE");
    }

    #[test]
    #[serial]
    fn test_deserialize_bool() {
        env::set_var("TEST_BOOL_TRUE", "true");
        env::set_var("TEST_BOOL_FALSE", "false");

        let t: bool = deserialize_required("TEST_BOOL_TRUE", false, None).unwrap();
        let f: bool = deserialize_required("TEST_BOOL_FALSE", false, None).unwrap();

        assert!(t);
        assert!(!f);

        env::remove_var("TEST_BOOL_TRUE");
        env::remove_var("TEST_BOOL_FALSE");
    }

    #[test]
    #[serial]
    fn test_deserialize_string() {
        env::set_var("TEST_STRING", "hello world");
        let result: String = deserialize_required("TEST_STRING", false, None).unwrap();
        assert_eq!(result, "hello world");
        env::remove_var("TEST_STRING");
    }

    #[test]
    #[serial]
    fn test_deserialize_url() {
        env::set_var("TEST_URL", "https://example.com/path?query=value");
        let result: String = deserialize_required("TEST_URL", false, None).unwrap();
        assert_eq!(result, "https://example.com/path?query=value");
        env::remove_var("TEST_URL");
    }

    #[test]
    #[serial]
    fn test_deserialize_optional_with_value() {
        env::set_var("TEST_OPT", "hello");
        let result: Option<String> = deserialize_optional("TEST_OPT", false, None).unwrap();
        assert_eq!(result, Some("hello".to_string()));
        env::remove_var("TEST_OPT");
    }

    #[test]
    #[serial]
    fn test_deserialize_optional_missing() {
        env::remove_var("TEST_OPT_MISSING");
        let result: Option<String> = deserialize_optional("TEST_OPT_MISSING", false, None).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    #[serial]
    fn test_get_env_value_file_not_found() {
        env::remove_var("TEST_FILE_MISSING");
        env::set_var("TEST_FILE_MISSING_FILE", "/nonexistent/file/path");

        let result = get_env_value("TEST_FILE_MISSING", true, None);
        assert!(matches!(result, Err(ServiceConfError::FileRead { .. })));

        env::remove_var("TEST_FILE_MISSING_FILE");
    }

    #[test]
    #[serial]
    fn test_parse_error_contains_type_info() {
        env::set_var("TEST_PARSE_ERR", "not_a_number");
        let result: Result<u32, _> = deserialize_required("TEST_PARSE_ERR", false, None);

        match result {
            Err(ServiceConfError::Parse { type_name, .. }) => {
                assert!(type_name.contains("u32"));
            }
            _ => panic!("Expected Parse error"),
        }

        env::remove_var("TEST_PARSE_ERR");
    }

    #[test]
    #[serial]
    fn test_get_env_value_with_config_dir() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().to_str().unwrap();
        
        // Create a test file in the config directory
        let file_path = temp_dir.path().join("TEST_CONFIG_VAR");
        fs::write(&file_path, "config_dir_value").unwrap();

        // Make sure these are not set
        env::remove_var("TEST_CONFIG_VAR");
        env::remove_var("TEST_CONFIG_VAR_FILE");

        let result = get_env_value("TEST_CONFIG_VAR", true, Some(config_dir)).unwrap();
        assert_eq!(result, "config_dir_value");
    }

    #[test]
    #[serial]
    fn test_config_dir_priority() {
        use std::fs;
        use std::io::Write;
        use tempfile::{NamedTempFile, TempDir};

        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().to_str().unwrap();
        
        // Create file in config directory
        let config_file_path = temp_dir.path().join("TEST_PRIORITY");
        fs::write(&config_file_path, "config_dir_value").unwrap();

        // Create separate file for _FILE env var
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "file_var_value").unwrap();

        // Test priority: direct env var > _FILE env var > config_dir
        
        // 1. Direct env var should win
        env::set_var("TEST_PRIORITY", "direct_value");
        env::set_var("TEST_PRIORITY_FILE", temp_file.path());
        let result = get_env_value("TEST_PRIORITY", true, Some(config_dir)).unwrap();
        assert_eq!(result, "direct_value");

        // 2. _FILE env var should win over config_dir
        env::remove_var("TEST_PRIORITY");
        let result = get_env_value("TEST_PRIORITY", true, Some(config_dir)).unwrap();
        assert_eq!(result, "file_var_value");

        // 3. config_dir should be used as fallback
        env::remove_var("TEST_PRIORITY_FILE");
        let result = get_env_value("TEST_PRIORITY", true, Some(config_dir)).unwrap();
        assert_eq!(result, "config_dir_value");

        env::remove_var("TEST_PRIORITY");
    }
}
