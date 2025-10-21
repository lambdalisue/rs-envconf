# serviceconf

Environment variable configuration with file-based secrets support

`serviceconf` provides a declarative API for loading configuration from environment
variables with native support for **file-based secrets** (Kubernetes Secrets, Docker Secrets).

The primary feature that distinguishes this library from other environment variable
configuration libraries is the `#[conf(from_file)]` attribute, which allows reading
secrets from files while falling back to direct environment variables for local development.

## Features

- **File-based secrets**: Read secrets from Kubernetes/Docker mounted files
- **Declarative**: Automatic implementation with `#[derive(ServiceConf)]`
- **Type-safe**: Compile-time type checking
- **Default values**: Support for `Default` trait and explicit values
- **Custom deserializers**: Support for JSON, TOML, or custom parsing functions

## Quick Start

```rust
use serviceconf::ServiceConf;

#[derive(Debug, ServiceConf)]
struct Config {
    // File-based secret: reads from API_KEY or API_KEY_FILE
    #[conf(from_file)]
    pub api_key: String,

    // Default value if not set
    #[conf(default = 8080)]
    pub port: u16,
}

let config = Config::from_env().unwrap();
println!("Port: {}", config.port);
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

## Key Feature: File-based Secrets

The `#[conf(from_file)]` attribute allows reading secrets from files mounted by
Kubernetes or Docker, avoiding the security risks of exposing secrets directly in
environment variables.

**Why file-based secrets?**
- ✅ **More secure**: Secrets stored in files, not environment variables (which can leak in logs, process lists, etc.)
- ✅ **Kubernetes native**: Works seamlessly with Kubernetes Secrets mounting
- ✅ **Docker Secrets**: Direct support for Docker Swarm secrets
- ✅ **Flexible**: Falls back to direct environment variables for local development

**Loading priority:**
1. Direct env var (`API_KEY`) - for local development
2. File path from env var (`API_KEY_FILE`) - for production

### Kubernetes Secret Example

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

With this Kubernetes configuration, your Rust application can simply use:

```rust
use serviceconf::ServiceConf;

#[derive(ServiceConf)]
struct Config {
    #[conf(from_file)]
    pub api_key: String,

    #[conf(from_file)]
    pub database_password: String,
}
```

## Value Parsing

**Default (using `FromStr`)**:
- Strings: `DATABASE_URL=postgres://localhost/db`
- Numbers: `MAX_CONNECTIONS=42`
- Booleans: `DEBUG=true`

**Custom deserializers** - specify with `#[conf(deserializer = "function")]`:
- JSON: `#[conf(deserializer = "serde_json::from_str")]`
- TOML: `#[conf(deserializer = "toml::from_str")]`
- Custom: Define your own deserializer function

## Attribute Reference

### Struct-level Attributes

#### `#[conf(prefix = "PREFIX_")]`

Add a prefix to all environment variable names in the struct.

```rust
use serviceconf::ServiceConf;

#[derive(ServiceConf)]
#[conf(prefix = "MYAPP_")]
struct Config {
    pub database_url: String,  // Reads from MYAPP_DATABASE_URL
    pub api_key: String,       // Reads from MYAPP_API_KEY
}
```

Environment variables:
```bash
export MYAPP_DATABASE_URL=postgres://localhost/db
export MYAPP_API_KEY=secret123
```

### Field-level Attributes

#### `#[conf(from_file)]` - File-based Secrets

Load from `{VAR_NAME}_FILE` in addition to the environment variable.
This is the primary feature of `serviceconf` for handling file-based secrets
in Kubernetes and Docker environments.

```rust
use serviceconf::ServiceConf;

#[derive(ServiceConf)]
pub struct Config {
    // Reads from API_KEY or API_KEY_FILE
    #[conf(from_file)]
    pub api_key: String,
}
```

#### `#[conf(name = "CUSTOM_NAME")]`

Specify an environment variable name different from the field name.

```rust
use serviceconf::ServiceConf;

#[derive(ServiceConf)]
pub struct Config {
    // Load from REDIS_URL environment variable
    #[conf(name = "REDIS_URL")]
    pub redis_connection_string: String,
}
```

#### `#[conf(default)]`

Use `Default::default()` if the environment variable is not set.

```rust
use serviceconf::ServiceConf;

#[derive(ServiceConf)]
struct Config {
    #[conf(default)]
    pub port: u16,  // Uses 0 (u16::default()) if PORT not set

    #[conf(default)]
    pub host: String,  // Uses "" (String::default()) if HOST not set
}
```

#### `#[conf(default = value)]`

Specify an explicit default value when the environment variable is not set.

```rust
use serviceconf::ServiceConf;

#[derive(ServiceConf)]
struct Config {
    #[conf(default = "127.0.0.1:8080".to_string())]
    pub server_addr: String,

    #[conf(default = 10)]
    pub max_connections: u32,

    #[conf(default = false)]
    pub enable_tls: bool,
}
```

#### `#[conf(deserializer = "function")]`

Use a custom deserializer function for complex types or custom parsing logic.

The deserializer function must have the signature:
```rust,ignore
fn deserialize(s: &str) -> Result<T, impl std::fmt::Display>
```

**Example with JSON:**
```rust
use serviceconf::ServiceConf;

#[derive(ServiceConf)]
struct Config {
    // Parse JSON array
    #[conf(deserializer = "serde_json::from_str")]
    pub tags: Vec<String>,
}
```

Environment variable:
```bash
export TAGS='["prod","api","v2"]'
```

**Example with custom function:**
```rust
use serviceconf::ServiceConf;

// Custom comma-separated parser
fn comma_separated(s: &str) -> Result<Vec<String>, String> {
    Ok(s.split(',').map(|s| s.trim().to_string()).collect())
}

#[derive(ServiceConf)]
struct Config {
    #[conf(deserializer = "comma_separated")]
    pub features: Vec<String>,
}
```

Environment variable:
```bash
export FEATURES=feature1,feature2,feature3
```

## Optional Fields

Use `Option<T>` for optional fields. Returns `None` if the environment variable is not set.

```rust
use serviceconf::ServiceConf;

#[derive(ServiceConf)]
struct Config {
    pub api_key: Option<String>,  // None if API_KEY not set
    pub max_retries: Option<u32>, // None if MAX_RETRIES not set
}
```

## Type Behavior

| Type | When Env Var Missing | When Env Var Set |
|------|---------------------|------------------|
| `T` (no attribute) | Error | Parsed with `FromStr` |
| `T` + `#[conf(default)]` | `Default::default()` | Parsed with `FromStr` |
| `T` + `#[conf(default = value)]` | Uses `value` | Parsed with `FromStr` |
| `Option<T>` | `None` | `Some(parsed_value)` |
| `T` + `#[conf(deserializer = "fn")]` | Error | Parsed with custom function |

## Combining Attributes

Multiple attributes can be combined to create powerful configurations:

```rust
use serviceconf::ServiceConf;

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
- `Option<T>` + `#[conf(default)]` or `#[conf(default = value)]` → Option already defaults to None

## Error Handling

The `from_env()` method returns a `Result` that can be handled appropriately:

```rust
use serviceconf::ServiceConf;

#[derive(ServiceConf)]
struct Config {
    pub api_key: String,
}

match Config::from_env() {
    Ok(config) => println!("Config loaded successfully"),
    Err(e) => eprintln!("Failed to load config: {}", e),
}
```

Example error messages:
- `Environment variable 'DATABASE_URL' is required but not set`
- `Failed to parse environment variable 'PORT' as u16: invalid digit found in string`
- `Failed to read file '/etc/secrets/key' for environment variable 'API_KEY_FILE': No such file or directory`
