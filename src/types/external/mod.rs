//! Implementations of `Type`, `ScalarType`, etc on external types.

mod bool;
mod char;
mod floats;
mod integers;
mod json_object;
mod list;
mod non_zero_integers;
mod optional;
mod string;
mod uuid;

#[cfg(feature = "bson")]
mod bson;
#[cfg(feature = "chrono_tz")]
mod chrono_tz;
#[cfg(feature = "chrono")]
mod datetime;
#[cfg(feature = "chrono")]
mod naive_time;
#[cfg(feature = "url")]
mod url;
