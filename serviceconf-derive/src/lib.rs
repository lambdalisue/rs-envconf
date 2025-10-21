#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

mod attrs;

use attrs::FieldAttrs;

/// Extract the inner type `T` from `Option<T>`, returning the original type if not an Option.
///
/// This helper is used to generate correct error messages and deserializer calls
/// for optional fields, where the inner type needs to be referenced separately.
fn extract_option_inner_type(ty: &Type) -> &Type {
    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                    return inner;
                }
            }
        }
    }
    ty
}

/// `ServiceConf` derive macro
///
/// Automatically implements the `from_env()` method on structs for loading configuration
/// from environment variables.
///
/// # Supported Attributes
///
/// ## Struct-level Attributes
///
/// ### `#[conf(prefix = "PREFIX_")]`
/// Add a prefix to all environment variable names in the struct.
///
/// ```no_run
/// use serviceconf::ServiceConf;
///
/// #[derive(ServiceConf)]
/// #[conf(prefix = "MYAPP_")]
/// struct Config {
///     pub api_key: String,  // Reads from MYAPP_API_KEY
///     pub port: u16,        // Reads from MYAPP_PORT
/// }
/// ```
///
/// ## Field-level Attributes
///
/// ### `#[conf(name = "CUSTOM_NAME")]`
/// Override the default environment variable name for a specific field.
///
/// ```no_run
/// use serviceconf::ServiceConf;
///
/// #[derive(ServiceConf)]
/// struct Config {
///     #[conf(name = "DATABASE_URL")]
///     pub db_connection: String,  // Reads from DATABASE_URL
/// }
/// ```
///
/// ### `#[conf(default)]`
/// Use `Default::default()` when the environment variable is not set.
///
/// ```no_run
/// use serviceconf::ServiceConf;
///
/// #[derive(ServiceConf)]
/// struct Config {
///     #[conf(default)]
///     pub port: u16,  // Uses 0 if PORT not set
/// }
/// ```
///
/// ### `#[conf(default = value)]`
/// Use an explicit default value when the environment variable is not set.
///
/// ```no_run
/// use serviceconf::ServiceConf;
///
/// #[derive(ServiceConf)]
/// struct Config {
///     #[conf(default = 8080)]
///     pub port: u16,  // Uses 8080 if PORT not set
/// }
/// ```
///
/// ### `#[conf(from_file)]`
/// Support loading from file-based secrets (Kubernetes/Docker Secrets).
/// Reads from both `VAR_NAME` and `VAR_NAME_FILE` environment variables.
///
/// ```no_run
/// use serviceconf::ServiceConf;
///
/// #[derive(ServiceConf)]
/// struct Config {
///     #[conf(from_file)]
///     pub api_key: String,  // Reads from API_KEY or API_KEY_FILE
/// }
/// ```
///
/// ### `#[conf(deserializer = "function")]`
/// Use a custom deserializer function for complex types.
///
/// The function signature must be: `fn(&str) -> Result<T, impl std::fmt::Display>`
///
/// ```no_run
/// use serviceconf::ServiceConf;
///
/// fn parse_list(s: &str) -> Result<Vec<String>, String> {
///     Ok(s.split(',').map(|s| s.trim().to_string()).collect())
/// }
///
/// #[derive(ServiceConf)]
/// struct Config {
///     #[conf(deserializer = "parse_list")]
///     pub items: Vec<String>,
/// }
/// ```
///
/// # Examples
///
/// **Basic usage:**
/// ```no_run
/// use serviceconf::ServiceConf;
///
/// #[derive(ServiceConf)]
/// struct Config {
///     pub api_key: String,
///
///     #[conf(default = 8080)]
///     pub port: u16,
/// }
///
/// fn main() -> anyhow::Result<()> {
///     let config = Config::from_env()?;
///     Ok(())
/// }
/// ```
///
/// **With prefix and file-based secrets:**
/// ```no_run
/// use serviceconf::ServiceConf;
///
/// #[derive(ServiceConf)]
/// #[conf(prefix = "APP_")]
/// struct Config {
///     #[conf(from_file)]
///     pub database_password: String,  // Reads from APP_DATABASE_PASSWORD or APP_DATABASE_PASSWORD_FILE
///
///     #[conf(default = 3000)]
///     pub port: u16,  // Reads from APP_PORT, defaults to 3000
/// }
/// ```
///
/// For complete documentation and more examples, see the [`serviceconf`](https://docs.rs/serviceconf) crate.
#[proc_macro_derive(ServiceConf, attributes(conf))]
pub fn derive_serviceconf(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Struct name
    let struct_name = &input.ident;

    // Parse struct-level attributes (prefix)
    let mut prefix = String::new();

    for attr in &input.attrs {
        if !attr.path().is_ident("conf") {
            continue;
        }

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("prefix") {
                let value = meta.value()?;
                let lit: syn::Lit = value.parse()?;
                if let syn::Lit::Str(s) = lit {
                    prefix = s.value();
                }
                return Ok(());
            }

            Err(meta.error("unsupported struct-level conf attribute"))
        });
    }

    // Extract fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "ServiceConf only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "ServiceConf only supports structs")
                .to_compile_error()
                .into();
        }
    };

    // Generate deserialization code for each field
    let field_initializers = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        // Parse attributes
        let attrs = FieldAttrs::from_field(field);

        // Check if type is Option<T>
        let is_option = if let syn::Type::Path(type_path) = field_type {
            type_path.path.segments.last()
                .map(|seg| seg.ident == "Option")
                .unwrap_or(false)
        } else {
            false
        };

        // Determine environment variable name
        let base_name = attrs.name.unwrap_or_else(|| {
            // Convert field name to UPPER_SNAKE_CASE
            field_name.to_string().to_uppercase()
        });

        // Apply prefix
        let env_var_name = format!("{}{}", prefix, base_name);

        let load_from_file = attrs.from_file;
        let deserializer_fn = attrs.deserializer;

        // Check for invalid combinations
        if is_option && attrs.default.is_some() {
            return syn::Error::new_spanned(
                field,
                "Option<T> fields cannot have default attribute (they default to None automatically)"
            )
            .to_compile_error();
        }

        // Generate deserialization expression
        let deserialize_expr = if is_option && deserializer_fn.is_none() {
            // Option<T> without deserializer
            let inner_type = extract_option_inner_type(field_type);

            quote! {
                ::serviceconf::de::deserialize_optional::<#inner_type>(
                    #env_var_name,
                    #load_from_file
                )?
            }
        } else if let Some(func_path) = deserializer_fn {
            // Use custom deserializer function
            let func: proc_macro2::TokenStream = func_path.parse().unwrap();
            if attrs.default.is_some() {
                return syn::Error::new_spanned(
                    field,
                    "default value is not supported with deserializer attribute"
                )
                .to_compile_error();
            }

            if is_option {
                // Option<T> with deserializer
                let inner_type = extract_option_inner_type(field_type);

                quote! {
                    match ::serviceconf::de::get_env_value(#env_var_name, #load_from_file) {
                        Ok(__value) => Some(#func(&__value).map_err(|e| ::serviceconf::ServiceConfError::parse_error::<#inner_type>(#env_var_name, e))?),
                        Err(::serviceconf::ServiceConfError::Missing { .. }) => None,
                        Err(e) => return Err(e.into()),
                    }
                }
            } else {
                // Non-Option with deserializer
                quote! {
                    {
                        let __value = ::serviceconf::de::get_env_value(#env_var_name, #load_from_file)?;
                        #func(&__value).map_err(|e| ::serviceconf::ServiceConfError::parse_error::<#field_type>(#env_var_name, e))?
                    }
                }
            }
        } else {
            // Use FromStr deserialization (default)
            match attrs.default {
                Some(Some(default_value)) => {
                    // Explicit default value
                    quote! {
                        ::serviceconf::de::deserialize_with_default::<#field_type>(
                            #env_var_name,
                            #load_from_file,
                            #default_value
                        )?
                    }
                }
                Some(None) => {
                    // Use Default::default()
                    quote! {
                        ::serviceconf::de::deserialize_with_default::<#field_type>(
                            #env_var_name,
                            #load_from_file,
                            Default::default()
                        )?
                    }
                }
                None => {
                    // Required field
                    quote! {
                        ::serviceconf::de::deserialize_required::<#field_type>(
                            #env_var_name,
                            #load_from_file
                        )?
                    }
                }
            }
        };

        quote! {
            #field_name: #deserialize_expr
        }
    });

    // Generate from_env() method
    let expanded = quote! {
        impl #struct_name {
            /// Load configuration from environment variables
            ///
            /// # Errors
            ///
            /// - Required environment variables are not set
            /// - Environment variable values cannot be parsed into target types
            /// - File-based configuration fails to read files
            pub fn from_env() -> ::serviceconf::anyhow::Result<Self> {
                Ok(Self {
                    #(#field_initializers),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
