use std::borrow::{Borrow, BorrowMut};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

/// Original position of element in source code
#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Default, Hash)]
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

#[derive(Copy, Clone, Debug, Default)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

/// Represents the location of a AST node
#[derive(Clone, Debug, Copy, Default)]
#[allow(missing_docs)]
pub struct Spanned<T: ?Sized> {
    pub span: Span,
    pub node: T,
}

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.node.fmt(f)
    }
}

impl<T: Clone> Spanned<T> {
    #[inline]
    #[allow(missing_docs)]
    pub fn clone_inner(&self) -> T {
        self.node.clone()
    }
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node.eq(&other.node)
    }
}

impl<T: PartialOrd> PartialOrd for Spanned<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.node.partial_cmp(&other.node)
    }
}

impl<T: Ord> Ord for Spanned<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.node.cmp(&other.node)
    }
}

impl<T: Ord> Eq for Spanned<T> {}

impl<T: ?Sized> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.node
    }
}

impl<T: ?Sized> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.node
    }
}

impl<T: Hash> Hash for Spanned<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node.hash(state)
    }
}

impl Borrow<str> for Spanned<String> {
    fn borrow(&self) -> &str {
        self.node.as_str()
    }
}

impl BorrowMut<str> for Spanned<String> {
    fn borrow_mut(&mut self) -> &mut str {
        self.node.as_mut_str()
    }
}

impl<T> Spanned<T> {
    pub(crate) fn new(node: T, pair_span: pest::Span<'_>) -> Spanned<T> {
        let ((start_line, start_column), (end_line, end_column)) = (
            pair_span.start_pos().line_col(),
            pair_span.end_pos().line_col(),
        );
        Spanned {
            node,
            span: Span {
                start: Pos {
                    line: start_line,
                    column: start_column,
                },
                end: Pos {
                    line: end_line,
                    column: end_column,
                },
            },
        }
    }

    #[inline]
    pub(crate) fn into_inner(self) -> T {
        self.node
    }

    /// Get start position
    #[inline]
    pub fn position(&self) -> Pos {
        self.span.start
    }

    #[inline]
    pub(crate) fn pack<F: FnOnce(Self) -> R, R>(self, f: F) -> Spanned<R> {
        Spanned {
            span: self.span,
            node: f(self),
        }
    }
}
