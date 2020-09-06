//! A parser for GraphQL. Used in the [`async-graphql`](https://crates.io/crates/async-graphql)
//! crate.
//!
//! It uses the [pest](https://crates.io/crates/pest) crate to parse the input and then transforms
//! it into Rust types.
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown, clippy::find_map, clippy::filter_map, clippy::module_name_repetitions, clippy::wildcard_imports, clippy::enum_glob_use)]

use std::fmt;
use pest::RuleType;
use pest::error::LineColLocation;

pub use pos::{Pos, Positioned};
pub use parser::parse_query;

pub mod types;

mod pos;
mod parser;
mod utils;

/// Parser error.
#[derive(Debug, PartialEq)]
pub struct Error {
    /// The position at which the error occurred.
    pub pos: Pos,
    /// The error message.
    pub message: String,
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
                    LineColLocation::Pos((line, column)) | LineColLocation::Span((line, column), _) => Pos { line, column },
                }
            },
            message: err.to_string(),
        }
    }
}

/// An alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
