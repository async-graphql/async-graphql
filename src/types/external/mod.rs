//! Implementations of `Type`, `ScalarType`, etc on external types.

mod bool;
mod bytes;
mod char;
mod cow;
mod floats;
mod integers;
mod json_object;
mod list;
mod non_zero_integers;
mod optional;
mod string;

#[cfg(feature = "bson")]
mod bson;
#[cfg(feature = "chrono-tz")]
mod chrono_tz;
#[cfg(feature = "chrono")]
mod datetime;
#[cfg(feature = "decimal")]
mod decimal;
#[cfg(feature = "chrono-duration")]
mod duration;
#[cfg(feature = "chrono")]
mod naive_time;
#[cfg(feature = "secrecy")]
mod secrecy;
#[cfg(feature = "url")]
mod url;
#[cfg(feature = "uuid")]
mod uuid;
