# envconf

A declarative Rust library for loading configuration from environment variables with file-based secret support.

## Installation

```toml
[dependencies]
envconf = { git = "https://github.com/lambdalisue/rs-envconf" }
```

## Quick Start

```rust
use envconf::EnvConf;

#[derive(Debug, EnvConf)]
struct Config {
    pub database_url: String,
    pub port: u16,
}

fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    println!("{:?}", config);
    Ok(())
}
```

```bash
export DATABASE_URL=postgres://localhost/db
export PORT=5432
```

## Core Features

### 1. Required Fields

Fields without attributes are required. Missing environment variables cause an error.

```rust
#[derive(EnvConf)]
struct Config {
    pub api_key: String,  // Required: error if API_KEY not set
}
```

### 2. Optional Fields

Use `Option<T>` for optional fields. Returns `None` if not set.

```rust
#[derive(EnvConf)]
struct Config {
    pub api_key: Option<String>,  // None if API_KEY not set
    pub port: Option<u16>,         // None if PORT not set
}
```

```bash
# api_key will be None, port will be Some(8080)
export PORT=8080
```

### 3. Default Values

Use `#[env(default)]` for `Default::default()` or `#[env(default = value)]` for explicit values.

```rust
#[derive(EnvConf)]
struct Config {
    #[env(default)]
    pub host: String,  // "" if HOST not set

    #[env(default = 8080)]
    pub port: u16,  // 8080 if PORT not set

    #[env(default = "localhost".to_string())]
    pub server: String,  // "localhost" if SERVER not set
}
```

### 4. File-based Secrets

Use `#[env(from_file)]` to load from files (Kubernetes Secrets, Docker Secrets).

```rust
#[derive(EnvConf)]
struct Config {
    #[env(from_file)]
    pub api_key: String,  // Reads from API_KEY or API_KEY_FILE
}
```

Priority:
1. Direct env var (`API_KEY`)
2. File path from env var (`API_KEY_FILE`)

Kubernetes example:
```yaml
env:
  - name: API_KEY_FILE
    value: /etc/secrets/api-key
volumeMounts:
  - name: secrets
    mountPath: /etc/secrets
    readOnly: true
```

### 5. Prefix

Use `#[env(prefix = "...")]` at struct level to prefix all environment variables.

```rust
#[derive(EnvConf)]
#[env(prefix = "MYAPP_")]
struct Config {
    pub database_url: String,  // Reads from MYAPP_DATABASE_URL
    pub api_key: String,       // Reads from MYAPP_API_KEY
}
```

```bash
export MYAPP_DATABASE_URL=postgres://localhost/db
export MYAPP_API_KEY=secret123
```

### 6. Custom Environment Variable Names

Use `#[env(name = "...")]` to override the auto-generated name.

```rust
#[derive(EnvConf)]
struct Config {
    #[env(name = "POSTGRES_URL")]
    pub database_url: String,  // Reads from POSTGRES_URL, not DATABASE_URL
}
```

### 7. Custom Deserializers

Use `#[env(deserializer = "function")]` for complex types or custom parsing.

```rust
// Custom parser
fn comma_separated(s: &str) -> Result<Vec<String>, String> {
    Ok(s.split(',').map(|s| s.trim().to_string()).collect())
}

#[derive(EnvConf)]
struct Config {
    // JSON array
    #[env(deserializer = "serde_json::from_str")]
    pub tags: Vec<String>,

    // Comma-separated
    #[env(deserializer = "comma_separated")]
    pub features: Vec<String>,

    // TOML (requires toml crate)
    #[env(deserializer = "toml::from_str")]
    pub settings: MySettings,
}
```

```bash
export TAGS='["prod","api","v2"]'
export FEATURES=feature1,feature2,feature3
```

## Attribute Reference

### Struct-level Attributes

| Attribute | Description |
|-----------|-------------|
| `#[env(prefix = "PREFIX_")]` | Add prefix to all environment variable names |

### Field-level Attributes

| Attribute | Description | When to Use |
|-----------|-------------|-------------|
| `#[env(name = "VAR")]` | Override environment variable name | When field name differs from desired env var |
| `#[env(default)]` | Use `Default::default()` if not set | For optional fields with sensible defaults |
| `#[env(default = value)]` | Use explicit default value | When you need a specific default |
| `#[env(from_file)]` | Support `{VAR}_FILE` pattern | For secrets stored in files |
| `#[env(deserializer = "fn")]` | Use custom parser | For complex types (Vec, HashMap, etc.) |

### Type Behavior

| Type | When Env Var Missing | When Env Var Set |
|------|---------------------|------------------|
| `T` (no attribute) | Error | Parsed with `FromStr` |
| `T` + `#[env(default)]` | `Default::default()` | Parsed with `FromStr` |
| `T` + `#[env(default = value)]` | Uses `value` | Parsed with `FromStr` |
| `Option<T>` | `None` | `Some(parsed_value)` |
| `T` + `#[env(deserializer = "fn")]` | Error | Parsed with custom function |

## Combining Attributes

Multiple attributes can be combined:

```rust
#[derive(EnvConf)]
#[env(prefix = "APP_")]
struct Config {
    // Combines: prefix + custom name + from_file + Option
    #[env(name = "DB_URL")]
    #[env(from_file)]
    pub database_url: Option<String>,  // Reads from APP_DB_URL or APP_DB_URL_FILE

    // Combines: prefix + default
    #[env(default = 8080)]
    pub port: u16,  // Reads from APP_PORT, defaults to 8080
}
```

**Invalid combinations** (compile errors):
- `Option<T>` + `#[env(default)]` → Option already defaults to None
- `#[env(deserializer = "...")]` + `#[env(default)]` → Not supported

## Examples

See the [`examples/`](examples/) directory for complete working examples:

| Example | Features Demonstrated |
|---------|----------------------|
| [`basic.rs`](examples/basic.rs) | Required fields, explicit default values |
| [`optional_fields.rs`](examples/optional_fields.rs) | `Option<T>` for optional fields |
| [`default_trait.rs`](examples/default_trait.rs) | `#[env(default)]` using `Default` trait |
| [`prefix.rs`](examples/prefix.rs) | `#[env(prefix = "...")]` at struct level |
| [`file_based_secrets.rs`](examples/file_based_secrets.rs) | `#[env(from_file)]` for Kubernetes/Docker secrets |
| [`custom_names.rs`](examples/custom_names.rs) | `#[env(name = "...")]` for custom env var names |
| [`complex_types.rs`](examples/complex_types.rs) | `Vec`, `HashMap` with JSON deserializer |
| [`custom_deserialize_fn.rs`](examples/custom_deserialize_fn.rs) | Custom deserializer functions |
| [`comprehensive.rs`](examples/comprehensive.rs) | Multiple features combined |

Run with: `cargo run --example <name>`

## Error Handling

```rust
match Config::from_env() {
    Ok(config) => println!("Config: {:?}", config),
    Err(e) => eprintln!("Error: {}", e),
}
```

Example errors:
- `Environment variable 'DATABASE_URL' is required but not set`
- `Failed to parse environment variable 'PORT' as u16: invalid digit found in string`
- `Failed to read file '/etc/secrets/key' for environment variable 'API_KEY_FILE': No such file or directory`

## Testing

```bash
cargo test
```

## License

Licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT).

## Contribution

Contributions are welcome! Please feel free to submit a Pull Request.
