//! Attribute parsing for `#[conf(...)]` annotations.
//!
//! This module extracts and validates configuration attributes from struct fields
//! during macro expansion.

use syn::{Field, Lit};

/// Parsed `#[conf(...)]` attributes from a struct field.
///
/// Represents all configuration options that can be specified on individual fields
/// of a `ServiceConf`-derived struct.
#[derive(Debug, Default)]
pub struct FieldAttrs {
    /// Custom environment variable name override.
    ///
    /// If `None`, the field name is converted to UPPER_SNAKE_CASE.
    pub name: Option<String>,

    /// Default value strategy:
    /// - `None`: Field is required (no default)
    /// - `Some(None)`: Use `Default::default()`
    /// - `Some(Some(tokens))`: Use explicit token stream as default value
    pub default: Option<Option<proc_macro2::TokenStream>>,

    /// Enable `{VAR}_FILE` pattern for reading secrets from mounted files.
    pub from_file: bool,

    /// Custom deserializer function path (e.g., `"serde_json::from_str"`).
    ///
    /// When specified, bypasses `FromStr` and uses this function instead.
    pub deserializer: Option<String>,
}

impl FieldAttrs {
    /// Extract and parse `#[conf(...)]` attributes from a struct field.
    ///
    /// Silently ignores unrecognized attributes to allow other macros to process them.
    pub fn from_field(field: &Field) -> Self {
        let mut attrs = Self::default();

        for attr in &field.attrs {
            if !attr.path().is_ident("conf") {
                continue;
            }

            // Parse #[conf(...)] contents
            let _ = attr.parse_nested_meta(|meta| {
                // name = "..."
                if meta.path.is_ident("name") {
                    let value = meta.value()?;
                    let name: Lit = value.parse()?;
                    if let Lit::Str(s) = name {
                        attrs.name = Some(s.value());
                    }
                    return Ok(());
                }

                // default or default = value
                if meta.path.is_ident("default") {
                    if meta.input.peek(syn::Token![=]) {
                        // default = value - explicit value
                        let value = meta.value()?;
                        let tokens: proc_macro2::TokenStream = value.parse()?;
                        attrs.default = Some(Some(tokens));
                    } else {
                        // default - use Default::default()
                        attrs.default = Some(None);
                    }
                    return Ok(());
                }

                // from_file
                if meta.path.is_ident("from_file") {
                    attrs.from_file = true;
                    return Ok(());
                }

                // deserializer = "function::path"
                if meta.path.is_ident("deserializer") {
                    let value = meta.value()?;
                    let func: Lit = value.parse()?;
                    if let Lit::Str(s) = func {
                        attrs.deserializer = Some(s.value());
                    }
                    return Ok(());
                }

                Err(meta.error("unsupported conf attribute"))
            });
        }

        attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parse_name_attribute() {
        let field: Field = parse_quote! {
            #[conf(name = "CUSTOM_NAME")]
            pub field_name: String
        };

        let attrs = FieldAttrs::from_field(&field);
        assert_eq!(attrs.name, Some("CUSTOM_NAME".to_string()));
    }

    #[test]
    fn test_parse_default_string() {
        let field: Field = parse_quote! {
            #[conf(default = "default_value")]
            pub field_name: String
        };

        let attrs = FieldAttrs::from_field(&field);
        assert!(attrs.default.is_some());
    }

    #[test]
    fn test_parse_default_number() {
        let field: Field = parse_quote! {
            #[conf(default = 42)]
            pub field_name: i32
        };

        let attrs = FieldAttrs::from_field(&field);
        assert!(attrs.default.is_some());
    }

    #[test]
    fn test_parse_from_file() {
        let field: Field = parse_quote! {
            #[conf(from_file)]
            pub field_name: String
        };

        let attrs = FieldAttrs::from_field(&field);
        assert!(attrs.from_file);
    }

    #[test]
    fn test_parse_multiple_attributes() {
        let field: Field = parse_quote! {
            #[conf(name = "DB_URL", from_file)]
            pub database_url: String
        };

        let attrs = FieldAttrs::from_field(&field);
        assert_eq!(attrs.name, Some("DB_URL".to_string()));
        assert!(attrs.from_file);
    }

    #[test]
    fn test_parse_default_no_value() {
        let field: Field = parse_quote! {
            #[conf(default)]
            pub field_name: String
        };

        let attrs = FieldAttrs::from_field(&field);
        assert!(matches!(attrs.default, Some(None)));
    }

    #[test]
    fn test_parse_deserializer() {
        let field: Field = parse_quote! {
            #[conf(deserializer = "serde_json::from_str")]
            pub field_name: Vec<String>
        };

        let attrs = FieldAttrs::from_field(&field);
        assert_eq!(attrs.deserializer, Some("serde_json::from_str".to_string()));
    }
}
