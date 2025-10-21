//! Attribute parsing

use syn::{Field, Lit};

/// Field attributes
#[derive(Debug, Default)]
pub struct FieldAttrs {
    /// Custom environment variable name
    pub name: Option<String>,
    /// Default value: Some(Some(tokens)) = explicit value, Some(None) = use Default trait, None = required
    pub default: Option<Option<proc_macro2::TokenStream>>,
    /// File-based configuration support
    pub from_file: bool,
    /// Custom deserializer function path
    pub deserializer: Option<String>,
}

impl FieldAttrs {
    /// Parse attributes from a field
    pub fn from_field(field: &Field) -> Self {
        let mut attrs = Self::default();

        for attr in &field.attrs {
            if !attr.path().is_ident("env") {
                continue;
            }

            // Parse #[env(...)] contents
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

                Err(meta.error("unsupported env attribute"))
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
            #[env(name = "CUSTOM_NAME")]
            pub field_name: String
        };

        let attrs = FieldAttrs::from_field(&field);
        assert_eq!(attrs.name, Some("CUSTOM_NAME".to_string()));
    }

    #[test]
    fn test_parse_default_string() {
        let field: Field = parse_quote! {
            #[env(default = "default_value")]
            pub field_name: String
        };

        let attrs = FieldAttrs::from_field(&field);
        assert!(attrs.default.is_some());
    }

    #[test]
    fn test_parse_default_number() {
        let field: Field = parse_quote! {
            #[env(default = 42)]
            pub field_name: i32
        };

        let attrs = FieldAttrs::from_field(&field);
        assert!(attrs.default.is_some());
    }

    #[test]
    fn test_parse_from_file() {
        let field: Field = parse_quote! {
            #[env(from_file)]
            pub field_name: String
        };

        let attrs = FieldAttrs::from_field(&field);
        assert!(attrs.from_file);
    }

    #[test]
    fn test_parse_multiple_attributes() {
        let field: Field = parse_quote! {
            #[env(name = "DB_URL", from_file)]
            pub database_url: String
        };

        let attrs = FieldAttrs::from_field(&field);
        assert_eq!(attrs.name, Some("DB_URL".to_string()));
        assert!(attrs.from_file);
    }

    #[test]
    fn test_parse_default_no_value() {
        let field: Field = parse_quote! {
            #[env(default)]
            pub field_name: String
        };

        let attrs = FieldAttrs::from_field(&field);
        assert!(matches!(attrs.default, Some(None)));
    }

    #[test]
    fn test_parse_deserializer() {
        let field: Field = parse_quote! {
            #[env(deserializer = "serde_json::from_str")]
            pub field_name: Vec<String>
        };

        let attrs = FieldAttrs::from_field(&field);
        assert_eq!(attrs.deserializer, Some("serde_json::from_str".to_string()));
    }
}
