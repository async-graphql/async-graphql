//! A parser for GraphQL. Used in the [`async-graphql`](https://crates.io/crates/async-graphql)
//! crate.
//!
//! It uses the [pest](https://crates.io/crates/pest) crate to parse the input and then transforms
//! it into Rust types.
#![forbid(unsafe_code)]

use pest::error::LineColLocation;
use pest::RuleType;
use std::fmt;

pub use parse::{parse_query, parse_schema};
pub use pos::{Pos, Positioned};

pub mod types;

mod parse;
mod pos;

/// Parser error.
#[derive(Debug, PartialEq)]
pub struct Error {
    /// The position at which the error occurred.
    pub pos: Pos,
    /// The error message.
    pub message: String,
}

impl Error {
    /// Create a new error with the given position and message.
    pub fn new(message: impl Into<String>, pos: Pos) -> Self {
        Self {
            pos,
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

impl<R: RuleType> From<pest::error::Error<R>> for Error {
    fn from(err: pest::error::Error<R>) -> Self {
        Error {
            pos: {
                match err.line_col {
                    LineColLocation::Pos((line, column))
                    | LineColLocation::Span((line, column), _) => Pos { line, column },
                }
            },
            message: err.to_string(),
        }
    }
}

/// An alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
