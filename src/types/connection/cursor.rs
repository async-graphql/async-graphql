use std::{
    convert::Infallible,
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
};

use crate::ID;

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

impl CursorType for usize {
    type Error = ParseIntError;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }

    fn encode_cursor(&self) -> String {
        self.to_string()
    }
}

impl CursorType for i32 {
    type Error = ParseIntError;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }

    fn encode_cursor(&self) -> String {
        self.to_string()
    }
}

impl CursorType for i64 {
    type Error = ParseIntError;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }

    fn encode_cursor(&self) -> String {
        self.to_string()
    }
}

impl CursorType for String {
    type Error = Infallible;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        Ok(s.to_string())
    }

    fn encode_cursor(&self) -> String {
        self.clone()
    }
}

impl CursorType for ID {
    type Error = Infallible;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        Ok(s.to_string().into())
    }

    fn encode_cursor(&self) -> String {
        self.to_string()
    }
}

impl CursorType for f64 {
    type Error = ParseFloatError;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }

    fn encode_cursor(&self) -> String {
        self.to_string()
    }
}

impl CursorType for f32 {
    type Error = ParseFloatError;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }

    fn encode_cursor(&self) -> String {
        self.to_string()
    }
}

pub enum OpaqueCursorError {
    Base64(base64::DecodeError),
    Serde(serde_json::Error),
}

impl std::fmt::Display for OpaqueCursorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Base64(err) => err.fmt(f),
            Self::Serde(err) => err.fmt(f),
        }
    }
}

struct OpaqueCursor<A>(A);

impl<A: serde::Serialize + serde::de::DeserializeOwned> CursorType for OpaqueCursor<A> {
    type Error = OpaqueCursorError;

    fn decode_cursor(str: &str) -> Result<Self, Self::Error> {
        base64::decode_config(str, base64::URL_SAFE_NO_PAD)
            .map_err(OpaqueCursorError::Base64)
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(OpaqueCursorError::Serde))
            .map(OpaqueCursor)
    }

    fn encode_cursor(&self) -> String {
        let json = serde_json::to_string(&self.0).expect("Failed to serialize json cursor");

        base64::encode_config(json, base64::URL_SAFE_NO_PAD)
    }
}
