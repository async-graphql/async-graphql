use std::fmt::Display;

/// Cursor type
///
/// A custom scalar that serializes as a string.
/// <https://relay.dev/graphql/connections.htm#sec-Cursor>
pub trait CursorType: Sized {
    /// Error type for `decode_cursor`.
    type Error: Display;

    /// Decode cursor from string.
    fn decode_cursor(s: &str) -> Result<Self, Self::Error>;

    /// Encode cursor to string.
    fn encode_cursor(&self) -> String;
}

pub enum CursorTypeError {
    Base64(base64::DecodeError),
    Serde(serde_json::Error),
}

impl std::fmt::Display for CursorTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Base64(err) => err.fmt(f),
            Self::Serde(err) => err.fmt(f),
        }
    }
}

impl<A: serde::Serialize + serde::de::DeserializeOwned> CursorType for A {
    type Error = CursorTypeError;

    fn decode_cursor(str: &str) -> Result<Self, Self::Error> {
        base64::decode_config(str, base64::URL_SAFE_NO_PAD)
            .map_err(CursorTypeError::Base64)
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(CursorTypeError::Serde))
    }

    fn encode_cursor(&self) -> String {
        let json = serde_json::to_string(&self).expect("Failed to serialize json cursor");

        base64::encode_config(json, base64::URL_SAFE_NO_PAD)
    }
}
