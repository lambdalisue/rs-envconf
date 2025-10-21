# serviceconf

[![Build](https://github.com/lambdalisue/rs-serviceconf/actions/workflows/build.yml/badge.svg)](https://github.com/lambdalisue/rs-serviceconf/actions/workflows/build.yml)
[![Crates.io Version](https://img.shields.io/crates/v/serviceconf)](https://crates.io/crates/serviceconf)
[![docs.rs](https://img.shields.io/docsrs/serviceconf)](https://docs.rs/serviceconf/0.1.0/serviceconf/)

**Environment variable configuration with file-based secrets support**

Load configuration from environment variables with native support for **file-based secrets** (Kubernetes Secrets, Docker Secrets). This is the primary feature that distinguishes `serviceconf` from other environment variable configuration libraries.

## Key Feature: File-based Secrets

The `#[conf(from_file)]` attribute allows reading secrets from files mounted by Kubernetes or Docker, avoiding the security risks of exposing secrets directly in environment variables:

```rust
use serviceconf::ServiceConf;

#[derive(ServiceConf)]
struct Config {
    #[conf(from_file)]
    pub api_key: String,  // Reads from API_KEY or API_KEY_FILE

    #[conf(from_file)]
    pub database_password: String,
}
```

**Why file-based secrets?**
- ✅ **More secure**: Secrets stored in files, not environment variables (which can leak in logs, process lists, etc.)
- ✅ **Kubernetes native**: Works seamlessly with Kubernetes Secrets mounting
- ✅ **Docker Secrets**: Direct support for Docker Swarm secrets
- ✅ **Flexible**: Falls back to direct environment variables for local development

**Loading priority:**
1. Direct env var (`API_KEY`) - for local development
2. File path from env var (`API_KEY_FILE`) - for production

**Kubernetes Secret example:**

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
type: Opaque
stringData:
  api-key: "prod-api-key-123"
  db-password: "secure-password"
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myservice
spec:
  template:
    spec:
      containers:
      - name: app
        image: myservice:latest
        env:
          - name: API_KEY_FILE
            value: /etc/secrets/api-key
          - name: DATABASE_PASSWORD_FILE
            value: /etc/secrets/db-password
        volumeMounts:
          - name: secrets
            mountPath: /etc/secrets
            readOnly: true
      volumes:
        - name: secrets
          secret:
            secretName: app-secrets
            items:
              - key: api-key
                path: api-key
              - key: db-password
                path: db-password
```

**Local development** (no files needed):
```bash
export API_KEY=dev-key-123
export DATABASE_PASSWORD=dev-password
```

## Installation

```toml
[dependencies]
serviceconf = { git = "https://github.com/lambdalisue/rs-serviceconf" }
```

## Quick Start

```rust
use serviceconf::ServiceConf;

#[derive(Debug, ServiceConf)]
struct Config {
    #[conf(from_file)]
    pub api_key: String,

    #[conf(default = 8080)]
    pub port: u16,
}

let config = Config::from_env()?;
println!("Port: {}", config.port);
# Ok::<(), Box<dyn std::error::Error>>(())
```

**Local development** (direct environment variable):
```bash
export API_KEY=dev-key-123
export PORT=3000
```

**Production** (Kubernetes/Docker with file-based secret):
```bash
export API_KEY_FILE=/run/secrets/api-key
export PORT=8080
```

## Other Features

### Default Values

Use `#[conf(default)]` for `Default::default()` or `#[conf(default = value)]` for explicit values.

```rust
#[derive(ServiceConf)]
struct Config {
    #[conf(default = 8080)]
    pub port: u16,  // 8080 if PORT not set

    #[conf(default = "localhost".to_string())]
    pub host: String,  // "localhost" if HOST not set
}
```

### Optional Fields

Use `Option<T>` for optional fields. Returns `None` if not set.

```rust
#[derive(ServiceConf)]
struct Config {
    pub api_key: Option<String>,  // None if API_KEY not set
}
```

### Prefix

Use `#[conf(prefix = "...")]` at struct level to prefix all environment variables.

```rust
#[derive(ServiceConf)]
#[conf(prefix = "MYAPP_")]
struct Config {
    pub database_url: String,  // Reads from MYAPP_DATABASE_URL
    pub api_key: String,       // Reads from MYAPP_API_KEY
}
```

```bash
export MYAPP_DATABASE_URL=postgres://localhost/db
export MYAPP_API_KEY=secret123
```

### Custom Environment Variable Names

Use `#[conf(name = "...")]` to override the auto-generated name.

```rust
#[derive(ServiceConf)]
struct Config {
    #[conf(name = "POSTGRES_URL")]
    pub database_url: String,  // Reads from POSTGRES_URL, not DATABASE_URL
}
```

### Custom Deserializers

Use `#[conf(deserializer = "function")]` for complex types or custom parsing.

```rust
// Custom parser
fn comma_separated(s: &str) -> Result<Vec<String>, String> {
    Ok(s.split(',').map(|s| s.trim().to_string()).collect())
}

#[derive(ServiceConf)]
struct Config {
    // JSON array
    #[conf(deserializer = "serde_json::from_str")]
    pub tags: Vec<String>,

    // Comma-separated
    #[conf(deserializer = "comma_separated")]
    pub features: Vec<String>,

    // TOML (requires toml crate)
    #[conf(deserializer = "toml::from_str")]
    pub settings: MySettings,
}
```

```bash
export TAGS='["prod","api","v2"]'
export FEATURES=feature1,feature2,feature3
```

## Attribute Reference

### Struct-level Attributes

| Attribute                    | Description                                  |
| ---------------------------- | -------------------------------------------- |
| `#[conf(prefix = "PREFIX_")]` | Add prefix to all environment variable names |

### Field-level Attributes

| Attribute                     | Description                         | When to Use                                  |
| ----------------------------- | ----------------------------------- | -------------------------------------------- |
| `#[conf(name = "VAR")]`        | Override environment variable name  | When field name differs from desired env var |
| `#[conf(default)]`             | Use `Default::default()` if not set | For optional fields with sensible defaults   |
| `#[conf(default = value)]`     | Use explicit default value          | When you need a specific default             |
| `#[conf(from_file)]`           | Support `{VAR}_FILE` pattern        | For secrets stored in files                  |
| `#[conf(deserializer = "fn")]` | Use custom parser                   | For complex types (Vec, HashMap, etc.)       |

### Type Behavior

| Type                                | When Env Var Missing | When Env Var Set            |
| ----------------------------------- | -------------------- | --------------------------- |
| `T` (no attribute)                  | Error                | Parsed with `FromStr`       |
| `T` + `#[conf(default)]`             | `Default::default()` | Parsed with `FromStr`       |
| `T` + `#[conf(default = value)]`     | Uses `value`         | Parsed with `FromStr`       |
| `Option<T>`                         | `None`               | `Some(parsed_value)`        |
| `T` + `#[conf(deserializer = "fn")]` | Error                | Parsed with custom function |

## Combining Attributes

Multiple attributes can be combined:

```rust
#[derive(ServiceConf)]
#[conf(prefix = "APP_")]
struct Config {
    // Combines: prefix + custom name + from_file + Option
    #[conf(name = "DB_URL")]
    #[conf(from_file)]
    pub database_url: Option<String>,  // Reads from APP_DB_URL or APP_DB_URL_FILE

    // Combines: prefix + default
    #[conf(default = 8080)]
    pub port: u16,  // Reads from APP_PORT, defaults to 8080
}
```

**Invalid combinations** (compile errors):

- `Option<T>` + `#[conf(default)]` → Option already defaults to None
- `#[conf(deserializer = "...")]` + `#[conf(default)]` → Not supported

## Examples

See the [`examples/`](examples/) directory for complete working examples:

| Example                                                         | Features Demonstrated                             |
| --------------------------------------------------------------- | ------------------------------------------------- |
| [`basic.rs`](examples/basic.rs)                                 | Required fields, explicit default values          |
| [`optional_fields.rs`](examples/optional_fields.rs)             | `Option<T>` for optional fields                   |
| [`default_trait.rs`](examples/default_trait.rs)                 | `#[conf(default)]` using `Default` trait           |
| [`prefix.rs`](examples/prefix.rs)                               | `#[conf(prefix = "...")]` at struct level          |
| [`file_based_secrets.rs`](examples/file_based_secrets.rs)       | `#[conf(from_file)]` for Kubernetes/Docker secrets |
| [`custom_names.rs`](examples/custom_names.rs)                   | `#[conf(name = "...")]` for custom env var names   |
| [`complex_types.rs`](examples/complex_types.rs)                 | `Vec`, `HashMap` with JSON deserializer           |
| [`custom_deserialize_fn.rs`](examples/custom_deserialize_fn.rs) | Custom deserializer functions                     |
| [`comprehensive.rs`](examples/comprehensive.rs)                 | Multiple features combined                        |

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
