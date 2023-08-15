use std::{
    char::ParseCharError,
    convert::Infallible,
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
    ops::{Deref, DerefMut},
    str::ParseBoolError,
};

use serde::{de::DeserializeOwned, Serialize};

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

macro_rules! cursor_type_int_impl {
    ($($t:ty)*) => {$(
        impl CursorType for $t {
            type Error = ParseIntError;

            fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
                s.parse()
            }

            fn encode_cursor(&self) -> String {
                self.to_string()
            }
        }
    )*}
}

cursor_type_int_impl! { isize i8 i16 i32 i64 i128 usize u8 u16 u32 u64 u128 }

impl CursorType for f32 {
    type Error = ParseFloatError;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        s.parse()
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

impl CursorType for char {
    type Error = ParseCharError;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }

    fn encode_cursor(&self) -> String {
        self.to_string()
    }
}

impl CursorType for bool {
    type Error = ParseBoolError;

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

#[cfg(feature = "chrono")]
impl CursorType for chrono::DateTime<chrono::Utc> {
    type Error = chrono::ParseError;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        Ok(chrono::DateTime::parse_from_rfc3339(s)?.with_timezone::<chrono::Utc>(&chrono::Utc {}))
    }

    fn encode_cursor(&self) -> String {
        self.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    }
}

#[cfg(feature = "uuid")]
impl CursorType for uuid::Uuid {
    type Error = uuid::Error;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }

    fn encode_cursor(&self) -> String {
        self.to_string()
    }
}

/// A opaque cursor that encode/decode the value to base64
pub struct OpaqueCursor<T>(pub T);

impl<T> Deref for OpaqueCursor<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for OpaqueCursor<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> CursorType for OpaqueCursor<T>
where
    T: Serialize + DeserializeOwned,
{
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        let data = base64::decode_config(s, base64::URL_SAFE_NO_PAD)?;
        Ok(Self(serde_json::from_slice(&data)?))
    }

    fn encode_cursor(&self) -> String {
        let value = serde_json::to_vec(&self.0).unwrap_or_default();
        base64::encode_config(value, base64::URL_SAFE_NO_PAD)
    }
}
