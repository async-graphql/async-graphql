//! Implementations of `Type`, `ScalarType`, etc on external types.

mod bool;
mod datetime;
mod floats;
mod integers;
mod list;
mod optional;
mod string;
mod url;
mod uuid;

#[cfg(feature = "bson")]
mod bson;
#[cfg(feature = "chrono_tz")]
mod chrono_tz;
