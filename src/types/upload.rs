use crate::{registry, InputValueType, Type, Value};
use std::borrow::Cow;

/// Upload file type
///
/// Reference: https://github.com/jaydenseric/graphql-multipart-request-spec
pub struct Upload {
    pub filename: String,
    pub content_type: Option<String>,
    pub content: Vec<u8>,
}

impl<'a> Type for Upload {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("Upload")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|_| registry::Type::Scalar {
            name: Self::type_name().to_string(),
            description: None,
            is_valid: |value| match value {
                Value::String(s) => s.starts_with("file:"),
                _ => false,
            },
        })
    }
}

impl<'a> InputValueType for Upload {
    fn parse(value: &Value) -> Option<Self> {
        if let Value::String(s) = value {
            if s.starts_with("file:") {
                let s = &s[5..];
                if let Some(idx) = s.find("|") {
                    let name_and_type = &s[..idx];
                    let content = &s[idx + 1..];
                    if let Some(type_idx) = name_and_type.find(":") {
                        let name = &name_and_type[..type_idx];
                        let mime_type = &name_and_type[type_idx + 1..];
                        return Some(Self {
                            filename: name.to_string(),
                            content_type: Some(mime_type.to_string()),
                            content: content.as_bytes().to_vec(),
                        });
                    } else {
                        return Some(Self {
                            filename: name_and_type.to_string(),
                            content_type: None,
                            content: content.as_bytes().to_vec(),
                        });
                    }
                }
            }
        }
        None
    }
}
