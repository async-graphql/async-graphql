use crate::{registry, InputValueError, InputValueResult, InputValueType, Type, Value};
use async_graphql_parser::UploadValue;
use std::borrow::Cow;
use std::io::Read;

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
///         println!("upload: filename={}", file.filename());
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
pub struct Upload(UploadValue);

impl Upload {
    /// Filename
    pub fn filename(&self) -> &str {
        self.0.filename.as_str()
    }

    /// Content type, such as `application/json`, `image/jpg` ...
    pub fn content_type(&self) -> Option<&str> {
        self.0.content_type.as_deref()
    }

    /// Convert to a read
    pub fn into_read(self) -> impl Read + Sync + Send + 'static {
        self.0.content
    }
}

impl<'a> Type for Upload {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("Upload")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|_| registry::MetaType::Scalar {
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
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::Upload(upload) = value {
            Ok(Upload(upload))
        } else {
            Err(InputValueError::ExpectedType(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::Null
    }
}
