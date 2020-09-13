//! Implementations of `Type`, `ScalarType`, etc on external types.

mod string;
mod list;
mod optional;
mod bool;
mod integers;
mod floats;
mod url;
mod uuid;
mod datetime;

#[cfg(feature = "bson")]
mod bson;
#[cfg(feature = "chrono_tz")]
mod chrono_tz;
