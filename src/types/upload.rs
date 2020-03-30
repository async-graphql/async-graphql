use crate::{registry, InputValueType, Type, Value};
use std::borrow::Cow;

/// Uploaded file
///
/// **Reference:** <https://github.com/jaydenseric/graphql-multipart-request-spec>
///
///
/// Graphql supports file uploads via `multipart/form-data`.
/// Enable this feature by accepting an argument of type `Upload` (single file) or
/// `Vec<Upload>` (multiple files) in your mutation like in the example blow.
///
///
/// # Example
/// *[Full Example](<https://github.com/sunli829/async-graphql/blob/master/async-graphql-actix-web/examples/upload-file.rs>)*
///
/// ```
/// use async_graphql::Upload;
///
/// struct MutationRoot;
///
/// #[async_graphql::Object]
/// impl MutationRoot {
///     #[field]
///     async fn upload(&self, file: Upload) -> bool {
///         println!(
///             "upload: filename={} size={}",
///             file.filename,
///             file.content.len()
///         );
///         true
///     }
/// }
///
/// ```
/// # Example Curl Request
/// Assuming you have defined your MutationRoot like in the example above,
/// you can now upload a file `myFile.txt` with the below curl command:
///
/// ```curl
/// curl POST 'localhost:8000' \
/// --form 'operations={
///         "query": "mutation ($file: Upload!) { upload(file: $file)  }",
///         "variables": { "file": null }}' \
/// --form 'map={ "0": ["variables.file"] }' \
/// --form '0=@myFile.txt'
/// ```
pub struct Upload {
    /// Filename
    pub filename: String,

    /// Content type, such as `application/json`, `image/jpg` ...
    pub content_type: Option<String>,

    /// File content
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
                if let Some(idx) = s.find('|') {
                    let name_and_type = &s[..idx];
                    let content = &s[idx + 1..];
                    if let Some(type_idx) = name_and_type.find(':') {
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
