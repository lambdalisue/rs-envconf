//! Deserialization functions for environment variables

use crate::error::EnvError;
use std::env;
use std::fs;
use std::str::FromStr;

/// Load a required value using `FromStr`
///
/// Used by the derive macro for fields without default values.
#[doc(hidden)]
pub fn deserialize_required<T>(env_name: &str, from_file: bool) -> Result<T, EnvError>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let value = get_env_value(env_name, from_file)?;
    value
        .parse::<T>()
        .map_err(|e| EnvError::parse_error::<T>(env_name, e))
}

/// Load a value with a default using `FromStr`
///
/// Used by the derive macro for fields with default values.
#[doc(hidden)]
pub fn deserialize_with_default<T>(
    env_name: &str,
    from_file: bool,
    default: T,
) -> Result<T, EnvError>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match get_env_value(env_name, from_file) {
        Ok(value) => value
            .parse::<T>()
            .map_err(|e| EnvError::parse_error::<T>(env_name, e)),
        Err(EnvError::Missing { .. }) => Ok(default),
        Err(e) => Err(e),
    }
}

/// Load an optional value using `FromStr`
///
/// Returns `None` if environment variable is not set, `Some(T)` if it is.
/// Used by the derive macro for `Option<T>` fields.
#[doc(hidden)]
pub fn deserialize_optional<T>(env_name: &str, from_file: bool) -> Result<Option<T>, EnvError>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match get_env_value(env_name, from_file) {
        Ok(value) => {
            let parsed = value
                .parse::<T>()
                .map_err(|e| EnvError::parse_error::<T>(env_name, e))?;
            Ok(Some(parsed))
        }
        Err(EnvError::Missing { .. }) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Get environment variable value with optional file-based fallback
///
/// Priority order:
/// 1. Direct environment variable (`env_name`)
/// 2. File from environment variable (`{env_name}_FILE`) if `from_file` is true
/// 3. Error if neither is found
///
/// Used by macro-generated code.
#[doc(hidden)]
pub fn get_env_value(env_name: &str, from_file: bool) -> Result<String, EnvError> {
    if let Ok(value) = env::var(env_name) {
        return Ok(value);
    }

    if from_file {
        let file_var_name = format!("{}_FILE", env_name);
        if let Ok(file_path) = env::var(&file_var_name) {
            return fs::read_to_string(&file_path)
                .map(|s| s.trim().to_string())
                .map_err(|e| EnvError::FileRead {
                    name: file_var_name,
                    path: file_path,
                    source: e,
                });
        }
    }

    Err(EnvError::missing(env_name))
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
        let result: Result<i32, _> = deserialize_required("TEST_VAR", false);
        assert_eq!(result.unwrap(), 42);
        env::remove_var("TEST_VAR");
    }

    #[test]
    #[serial]
    fn test_deserialize_required_missing() {
        env::remove_var("MISSING_VAR");
        let result: Result<String, _> = deserialize_required("MISSING_VAR", false);
        assert!(matches!(result, Err(EnvError::Missing { .. })));
    }

    #[test]
    #[serial]
    fn test_deserialize_with_default_env_set() {
        env::set_var("TEST_DEFAULT", "100");
        let result: u32 = deserialize_with_default("TEST_DEFAULT", false, 50).unwrap();
        assert_eq!(result, 100);
        env::remove_var("TEST_DEFAULT");
    }

    #[test]
    #[serial]
    fn test_deserialize_with_default_use_default() {
        env::remove_var("TEST_DEFAULT_MISSING");
        let result: u32 = deserialize_with_default("TEST_DEFAULT_MISSING", false, 50).unwrap();
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

        let result = get_env_value("TEST_FILE_VAR", true).unwrap();
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

        let result = get_env_value("TEST_PREFER", true).unwrap();
        assert_eq!(result, "direct_value");

        env::remove_var("TEST_PREFER");
        env::remove_var("TEST_PREFER_FILE");
    }

    #[test]
    #[serial]
    fn test_deserialize_bool() {
        env::set_var("TEST_BOOL_TRUE", "true");
        env::set_var("TEST_BOOL_FALSE", "false");

        let t: bool = deserialize_required("TEST_BOOL_TRUE", false).unwrap();
        let f: bool = deserialize_required("TEST_BOOL_FALSE", false).unwrap();

        assert!(t);
        assert!(!f);

        env::remove_var("TEST_BOOL_TRUE");
        env::remove_var("TEST_BOOL_FALSE");
    }

    #[test]
    #[serial]
    fn test_deserialize_string() {
        env::set_var("TEST_STRING", "hello world");
        let result: String = deserialize_required("TEST_STRING", false).unwrap();
        assert_eq!(result, "hello world");
        env::remove_var("TEST_STRING");
    }

    #[test]
    #[serial]
    fn test_deserialize_url() {
        env::set_var("TEST_URL", "https://example.com/path?query=value");
        let result: String = deserialize_required("TEST_URL", false).unwrap();
        assert_eq!(result, "https://example.com/path?query=value");
        env::remove_var("TEST_URL");
    }

    #[test]
    #[serial]
    fn test_deserialize_optional_with_value() {
        env::set_var("TEST_OPT", "hello");
        let result: Option<String> = deserialize_optional("TEST_OPT", false).unwrap();
        assert_eq!(result, Some("hello".to_string()));
        env::remove_var("TEST_OPT");
    }

    #[test]
    #[serial]
    fn test_deserialize_optional_missing() {
        env::remove_var("TEST_OPT_MISSING");
        let result: Option<String> = deserialize_optional("TEST_OPT_MISSING", false).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    #[serial]
    fn test_get_env_value_file_not_found() {
        env::remove_var("TEST_FILE_MISSING");
        env::set_var("TEST_FILE_MISSING_FILE", "/nonexistent/file/path");

        let result = get_env_value("TEST_FILE_MISSING", true);
        assert!(matches!(result, Err(EnvError::FileRead { .. })));

        env::remove_var("TEST_FILE_MISSING_FILE");
    }

    #[test]
    #[serial]
    fn test_parse_error_contains_type_info() {
        env::set_var("TEST_PARSE_ERR", "not_a_number");
        let result: Result<u32, _> = deserialize_required("TEST_PARSE_ERR", false);

        match result {
            Err(EnvError::Parse { type_name, .. }) => {
                assert!(type_name.contains("u32"));
            }
            _ => panic!("Expected Parse error"),
        }

        env::remove_var("TEST_PARSE_ERR");
    }
}
