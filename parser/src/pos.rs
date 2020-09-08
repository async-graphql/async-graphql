use serde::Serialize;
use std::borrow::{Borrow, BorrowMut};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

/// Original position of element in source code
#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Default, Hash, Serialize)]
pub struct Pos {
    /// One-based line number
    pub line: usize,

    /// One-based column number
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

/// Represents the position of a AST node
#[derive(Clone, Debug, Copy, Default)]
#[allow(missing_docs)]
pub struct Positioned<T: ?Sized> {
    pub pos: Pos,
    pub node: T,
}

impl<T: fmt::Display> fmt::Display for Positioned<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.node.fmt(f)
    }
}

impl<T: Clone> Positioned<T> {
    #[inline]
    #[allow(missing_docs)]
    pub fn clone_inner(&self) -> T {
        self.node.clone()
    }
}

impl<T: PartialEq> PartialEq for Positioned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node.eq(&other.node)
    }
}

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

impl<T: Ord> Eq for Positioned<T> {}

impl<T: ?Sized> Deref for Positioned<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.node
    }
}

impl<T: ?Sized> DerefMut for Positioned<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.node
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

impl<T> Positioned<T> {
    pub(crate) fn new(node: T, pos: Pos) -> Positioned<T> {
        Positioned { node, pos }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.node
    }

    /// Get start position
    #[inline]
    pub fn position(&self) -> Pos {
        self.pos
    }

    #[inline]
    pub(crate) fn pack<F: FnOnce(Self) -> R, R>(self, f: F) -> Positioned<R> {
        Positioned {
            pos: self.pos,
            node: f(self),
        }
    }
}
