use std::collections::HashMap;

use crate::parser::types::{Field, FragmentDefinition, Selection, SelectionSet};
use crate::{Name, Positioned, SelectionField};

/// A selection performed by a query.
pub struct Lookahead<'a> {
    fragments: &'a HashMap<Name, Positioned<FragmentDefinition>>,
    field: Option<&'a Field>,
}

impl<'a> Lookahead<'a> {
    pub(crate) fn new(
        fragments: &'a HashMap<Name, Positioned<FragmentDefinition>>,
        field: &'a Field,
    ) -> Self {
        Self {
            fragments,
            field: Some(field),
        }
    }

    /// Get the first subfield of the selection set with the specified name. This will ignore
    /// aliases.
    ///
    /// For example, calling `.field("a")` on `{ a { b } }` will return a lookahead that
    /// represents `{ b }`.
    pub fn field(&self, name: &str) -> Self {
        Self {
            fragments: self.fragments,
            field: self
                .field
                .and_then(|field| find(self.fragments, &field.selection_set.node, name)),
        }
    }

    /// Returns true if field exists otherwise return false.
    #[inline]
    pub fn exists(&self) -> bool {
        self.field.is_some()
    }
}

impl<'a> From<SelectionField<'a>> for Lookahead<'a> {
    fn from(selection_field: SelectionField<'a>) -> Self {
        Lookahead {
            fragments: selection_field.fragments,
            field: Some(selection_field.field),
        }
    }
}

fn find<'a>(
    fragments: &'a HashMap<Name, Positioned<FragmentDefinition>>,
    selection_set: &'a SelectionSet,
    name: &str,
) -> Option<&'a Field> {
    selection_set
        .items
        .iter()
        .find_map(|item| match &item.node {
            Selection::Field(field) => {
                if field.node.name.node == name {
                    Some(&field.node)
                } else {
                    None
                }
            }
            Selection::InlineFragment(fragment) => {
                find(fragments, &fragment.node.selection_set.node, name)
            }
            Selection::FragmentSpread(spread) => fragments
                .get(&spread.node.fragment_name.node)
                .and_then(|fragment| find(fragments, &fragment.node.selection_set.node, name)),
        })
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
                } else if ctx.look_ahead().field("detail").field("c").exists() {
                    // This is a query like `obj { detail { c } }`
                    assert_eq!(n, 2);
                } else {
                    // This query doesn't have `a`
                    assert_eq!(n, 3);
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
            obj(n: 2) {
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
            obj(n: 3) {
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
            obj(n: 2) {
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
            obj(n: 2) {
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
    }
}
