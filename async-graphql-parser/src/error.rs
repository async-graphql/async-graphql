use crate::Pos;
use pest::error::LineColLocation;
use pest::RuleType;
use std::fmt;

/// Parser error
#[derive(Error, Debug, PartialEq)]
pub struct Error {
    pub pos: Pos,
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl<R: RuleType> From<pest::error::Error<R>> for Error {
    fn from(err: pest::error::Error<R>) -> Self {
        Error {
            pos: {
                let (line, column) = match err.line_col {
                    LineColLocation::Pos((line, column)) => (line, column),
                    LineColLocation::Span((line, column), _) => (line, column),
                };
                Pos { line, column }
            },
            message: err.to_string(),
        }
    }
}
