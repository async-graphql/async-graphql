use crate::Error;
use pest::iterators::Pair;
use pest::RuleType;
use serde::Serialize;
use std::borrow::{Borrow, BorrowMut};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::Chars;

/// Original position of an element in source code.
#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Default, Hash, Serialize)]
pub struct Pos {
    /// One-based line number.
    pub line: usize,

    /// One-based column number.
    pub column: usize,
}

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pos({}:{})", self.line, self.column)
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// An AST node that stores its original position.
#[derive(Debug, Clone, Copy, Default)]
pub struct Positioned<T: ?Sized> {
    /// The position of the node.
    pub pos: Pos,
    /// The node itself.
    pub node: T,
}

impl<T> Positioned<T> {
    /// Create a new positioned node from the node and its position.
    #[must_use]
    pub const fn new(node: T, pos: Pos) -> Positioned<T> {
        Positioned { node, pos }
    }

    /// Get the inner node.
    ///
    /// This is most useful in callback chains where `Positioned::into_inner` is easier to read than
    /// `|positioned| positioned.node`.
    #[inline]
    pub fn into_inner(self) -> T {
        self.node
    }

    /// Create a new positioned node with the same position as this one.
    #[must_use]
    pub fn position_node<U>(&self, other: U) -> Positioned<U> {
        Positioned::new(other, self.pos)
    }

    /// Map the inner value of this positioned node.
    #[must_use]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Positioned<U> {
        Positioned::new(f(self.node), self.pos)
    }

    pub(crate) fn error_here(&self, message: impl Into<String>) -> Error {
        Error::new(message, self.pos)
    }
}

impl<T: fmt::Display> fmt::Display for Positioned<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.node.fmt(f)
    }
}
impl<T: PartialEq> PartialEq for Positioned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}
impl<T: Eq> Eq for Positioned<T> {}
impl<T: PartialOrd> PartialOrd for Positioned<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.node.partial_cmp(&other.node)
    }
}
impl<T: Ord> Ord for Positioned<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.node.cmp(&other.node)
    }
}
impl<T: Hash> Hash for Positioned<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node.hash(state)
    }
}

impl Borrow<str> for Positioned<String> {
    fn borrow(&self) -> &str {
        self.node.as_str()
    }
}

impl BorrowMut<str> for Positioned<String> {
    fn borrow_mut(&mut self) -> &mut str {
        self.node.as_mut_str()
    }
}

pub(crate) struct PositionCalculator<'a> {
    input: Chars<'a>,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> PositionCalculator<'a> {
    pub(crate) fn new(input: &'a str) -> PositionCalculator<'a> {
        Self {
            input: input.chars(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    pub(crate) fn step<R: RuleType>(&mut self, pair: &Pair<R>) -> Pos {
        let pos = pair.as_span().start();
        debug_assert!(pos >= self.pos);
        for _ in 0..pos - self.pos {
            match self.input.next() {
                Some('\r') => {
                    self.column = 1;
                }
                Some('\n') => {
                    self.line += 1;
                    self.column = 1;
                }
                Some(_) => {
                    self.column += 1;
                }
                None => break,
            }
        }
        self.pos = pos;
        Pos {
            line: self.line,
            column: self.column,
        }
    }
}
