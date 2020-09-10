use crate::ID;
use std::convert::Infallible;
use std::fmt::Display;
use std::num::ParseIntError;

/// Cursor type
///
/// A custom scalar that serializes as a string.
/// https://relay.dev/graphql/connections.htm#sec-Cursor
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
