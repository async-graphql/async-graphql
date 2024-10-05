use indexmap::IndexMap;

use crate::{registry::MetaDirectiveInvocation, Value};

/// A GraphQL directive
#[derive(Debug, Clone)]
pub struct Directive {
    name: String,
    args: IndexMap<String, Value>,
}

impl Directive {
    /// Create a directive usage
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            args: IndexMap::default(),
        }
    }

    /// Add an argument to the directive
    #[inline]
    pub fn argument(mut self, name: impl Into<String>, value: Value) -> Self {
        self.args.insert(name.into(), value);
        self
    }
}

impl From<Directive> for MetaDirectiveInvocation {
    fn from(directive: Directive) -> Self {
        Self {
            name: directive.name,
            args: directive.args,
        }
    }
}

pub fn to_meta_directive_invocation(directives: Vec<Directive>) -> Vec<MetaDirectiveInvocation> {
    directives
        .into_iter()
        .map(MetaDirectiveInvocation::from)
        .collect()
}
