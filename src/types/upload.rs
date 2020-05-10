use crate::{registry, InputValueError, InputValueResult, InputValueType, Type, Value};
use std::borrow::Cow;
use std::path::PathBuf;

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
/// *[Full Example](<https://github.com/async-graphql/examples/blob/master/models/files/src/lib.rs>)*
///
/// ```
/// use async_graphql::Upload;
///
/// struct MutationRoot;
///
/// #[async_graphql::Object]
/// impl MutationRoot {
///     async fn upload(&self, file: Upload) -> bool {
///         println!("upload: filename={}", file.filename);
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
/// curl 'localhost:8000' \
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

    /// Temporary file path
    pub path: PathBuf,
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
    fn parse(value: &Value) -> InputValueResult<Self> {
        if let Value::String(s) = value {
            if s.starts_with("file:") {
                let s = &s[5..];
                if let Some(idx) = s.find('|') {
                    let name_and_type = &s[..idx];
                    let path = &s[idx + 1..];
                    if let Some(type_idx) = name_and_type.find(':') {
                        let name = &name_and_type[..type_idx];
                        let mime_type = &name_and_type[type_idx + 1..];
                        return Ok(Self {
                            filename: name.to_string(),
                            content_type: Some(mime_type.to_string()),
                            path: PathBuf::from(path),
                        });
                    } else {
                        return Ok(Self {
                            filename: name_and_type.to_string(),
                            content_type: None,
                            path: PathBuf::from(path),
                        });
                    }
                }
            }
        }
        Err(InputValueError::ExpectedType)
    }
}
