//! Derive macro implementation for serviceconf

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

mod attrs;

use attrs::FieldAttrs;

/// Extract inner type from Option<T>
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
/// Automatically implements the `from_env()` method on structs.
///
/// # Supported Attributes
///
/// **Struct-level**:
/// - `#[conf(prefix = "PREFIX_")]`: Add prefix to all env var names
///
/// **Field-level**:
/// - `#[conf(name = "CUSTOM_NAME")]`: Custom environment variable name
/// - `#[conf(default)]`: Use `Default::default()` if env var not set
/// - `#[conf(default = value)]`: Use explicit default value if env var not set
/// - `#[conf(from_file)]`: Support `{VAR}_FILE` pattern
/// - `#[conf(deserializer = "func")]`: Use custom deserializer function
///
/// # Example
///
/// See the `serviceconf` crate documentation for usage examples.
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
                        Ok(__value) => Some(#func(&__value).map_err(|e| ::serviceconf::EnvError::parse_error::<#inner_type>(#env_var_name, e))?),
                        Err(::serviceconf::EnvError::Missing { .. }) => None,
                        Err(e) => return Err(e.into()),
                    }
                }
            } else {
                // Non-Option with deserializer
                quote! {
                    {
                        let __value = ::serviceconf::de::get_env_value(#env_var_name, #load_from_file)?;
                        #func(&__value).map_err(|e| ::serviceconf::EnvError::parse_error::<#field_type>(#env_var_name, e))?
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
