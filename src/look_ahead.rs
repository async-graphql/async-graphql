use std::collections::HashMap;

use crate::parser::types::{Field, FragmentDefinition, Selection, SelectionSet};
use crate::{Name, Positioned, SelectionField};

/// A selection performed by a query.
pub struct Lookahead<'a> {
    fragments: &'a HashMap<Name, Positioned<FragmentDefinition>>,
    fields: Vec<&'a Field>,
}

impl<'a> Lookahead<'a> {
    pub(crate) fn new(
        fragments: &'a HashMap<Name, Positioned<FragmentDefinition>>,
        field: &'a Field,
    ) -> Self {
        Self {
            fragments,
            fields: vec![field],
        }
    }

    /// Get the field of the selection set with the specified name. This will ignore
    /// aliases.
    ///
    /// For example, calling `.field("a")` on `{ a { b } }` will return a lookahead that
    /// represents `{ b }`.
    pub fn field(&self, name: &str) -> Self {
        let mut fields = Vec::new();
        for field in &self.fields {
            filter(&mut fields, self.fragments, &field.selection_set.node, name)
        }

        Self {
            fragments: self.fragments,
            fields,
        }
    }

    /// Returns true if field exists otherwise return false.
    #[inline]
    pub fn exists(&self) -> bool {
        !self.fields.is_empty()
    }
}

impl<'a> From<SelectionField<'a>> for Lookahead<'a> {
    fn from(selection_field: SelectionField<'a>) -> Self {
        Lookahead {
            fragments: selection_field.fragments,
            fields: vec![selection_field.field],
        }
    }
}

/// Convert a slice of `SelectionField`s to a `Lookahead`.
///
/// This assumes that the `SelectionField`s are all from the same query.
///
/// # Panics
/// Panics if 0 `SelectionField`s are provided.
impl<'a> From<&[SelectionField<'a>]> for Lookahead<'a> {
    fn from(selection_fields: &[SelectionField<'a>]) -> Self {
        Lookahead {
            fragments: selection_fields[0].fragments,
            fields: selection_fields
                .iter()
                .map(|selection_field| selection_field.field)
                .collect(),
        }
    }
}

fn filter<'a>(
    fields: &mut Vec<&'a Field>,
    fragments: &'a HashMap<Name, Positioned<FragmentDefinition>>,
    selection_set: &'a SelectionSet,
    name: &str,
) {
    for item in &selection_set.items {
        // doing this imperatively is a bit nasty, but using iterators would
        // require a boxed return type (I believe) as its recusive
        match &item.node {
            Selection::Field(field) => {
                if field.node.name.node == name {
                    fields.push(&field.node)
                }
            }
            Selection::InlineFragment(fragment) => {
                filter(fields, fragments, &fragment.node.selection_set.node, name)
            }
            Selection::FragmentSpread(spread) => {
                if let Some(fragment) = fragments.get(&spread.node.fragment_name.node) {
                    filter(fields, fragments, &fragment.node.selection_set.node, name)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn test_look_ahead() {
        #[derive(SimpleObject)]
        #[graphql(internal)]
        struct Detail {
            c: i32,
            d: i32,
        }

        #[derive(SimpleObject)]
        #[graphql(internal)]
        struct MyObj {
            a: i32,
            b: i32,
            detail: Detail,
        }

        struct Query;

        #[Object(internal)]
        impl Query {
            async fn obj(&self, ctx: &Context<'_>, n: i32) -> MyObj {
                if ctx.look_ahead().field("a").exists() {
                    // This is a query like `obj { a }`
                    assert_eq!(n, 1);
                } else if ctx.look_ahead().field("detail").field("c").exists()
                    && ctx.look_ahead().field("detail").field("d").exists()
                {
                    // This is a query like `obj { detail { c } }`
                    assert_eq!(n, 2);
                } else if ctx.look_ahead().field("detail").field("c").exists() {
                    // This is a query like `obj { detail { c } }`
                    assert_eq!(n, 3);
                } else {
                    // This query doesn't have `a`
                    assert_eq!(n, 4);
                }
                MyObj {
                    a: 0,
                    b: 0,
                    detail: Detail { c: 0, d: 0 },
                }
            }
        }

        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

        assert!(schema
            .execute(
                r#"{
            obj(n: 1) {
                a
            }
        }"#,
            )
            .await
            .is_ok());

        assert!(schema
            .execute(
                r#"{
            obj(n: 1) {
                k:a
            }
        }"#,
            )
            .await
            .is_ok());

        assert!(schema
            .execute(
                r#"{
            obj(n: 3) {
                detail {
                    c
                }
            }
        }"#,
            )
            .await
            .is_ok());

        assert!(schema
            .execute(
                r#"{
            obj(n: 2) {
                detail {
                    d
                }

                detail {
                    c
                }
            }
        }"#,
            )
            .await
            .is_ok());

        assert!(schema
            .execute(
                r#"{
            obj(n: 4) {
                b
            }
        }"#,
            )
            .await
            .is_ok());

        assert!(schema
            .execute(
                r#"{
            obj(n: 1) {
                ... {
                    a
                }
            }
        }"#,
            )
            .await
            .is_ok());

        assert!(schema
            .execute(
                r#"{
            obj(n: 3) {
                ... {
                    detail {
                        c
                    }
                }
            }
        }"#,
            )
            .await
            .is_ok());

        assert!(schema
            .execute(
                r#"{
            obj(n: 2) {
                ... {
                    detail {
                        d
                    }

                    detail {
                        c
                    }
                }
            }
        }"#,
            )
            .await
            .is_ok());

        assert!(schema
            .execute(
                r#"{
            obj(n: 1) {
                ... A
            }
        }
        
        fragment A on MyObj {
            a
        }"#,
            )
            .await
            .is_ok());

        assert!(schema
            .execute(
                r#"{
            obj(n: 3) {
                ... A
            }
        }
        
        fragment A on MyObj {
            detail {
                c
            }
        }"#,
            )
            .await
            .is_ok());

        assert!(schema
            .execute(
                r#"{
            obj(n: 2) {
                ... A
                ... B
            }
        }
        
        fragment A on MyObj {
            detail {
                d
            }
        }
        
        fragment B on MyObj {
            detail {
                c
            }
        }"#,
            )
            .await
            .is_ok());
    }
}
