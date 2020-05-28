use crate::ID;
use byteorder::{ReadBytesExt, BE};
use std::convert::Infallible;
use std::fmt::Display;

/// Cursor type
///
/// A custom scalar that serializes as a string.
/// https://relay.dev/graphql/connections.htm#sec-Cursor
pub trait CursorType: Sized {
    /// Error type for `encode_cursor` and `decode_cursor`.
    type Error: Display;

    /// Decode cursor from string.
    fn decode_cursor(s: &str) -> Result<Self, Self::Error>;

    /// Encode cursor to string.
    fn encode_cursor(&self) -> Result<String, Self::Error>;
}

impl CursorType for usize {
    type Error = anyhow::Error;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        let data = base64::decode(s)?;
        Ok(data.as_slice().read_u32::<BE>()? as usize)
    }

    fn encode_cursor(&self) -> Result<String, Self::Error> {
        Ok(base64::encode((*self as u32).to_be_bytes()))
    }
}

impl CursorType for String {
    type Error = Infallible;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        Ok(s.to_string())
    }

    fn encode_cursor(&self) -> Result<String, Self::Error> {
        Ok(self.clone())
    }
}

impl CursorType for ID {
    type Error = Infallible;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        Ok(s.to_string().into())
    }

    fn encode_cursor(&self) -> Result<String, Self::Error> {
        Ok(self.to_string())
    }
}
