use crate::parser::types::{ExecutableDocumentData, Field, Selection, SelectionSet};

/// A selection performed by a query
pub struct Lookahead<'a> {
    pub(crate) document: &'a ExecutableDocumentData,
    pub(crate) field: Option<&'a Field>,
}

impl<'a> Lookahead<'a> {
    /// Check if the specified field exists in the current selection.
    pub fn field(&self, name: &str) -> Self {
        Self {
            document: self.document,
            field: self
                .field
                .and_then(|field| find(self.document, &field.selection_set.node, name)),
        }
    }

    /// Returns true if field exists otherwise return false.
    #[inline]
    pub fn exists(&self) -> bool {
        self.field.is_some()
    }
}

fn find<'a>(
    document: &'a ExecutableDocumentData,
    selection_set: &'a SelectionSet,
    name: &str,
) -> Option<&'a Field> {
    for item in &selection_set.items {
        match &item.node {
            Selection::Field(field) => {
                if field.node.name.node == name {
                    return Some(&field.node);
                }
            }
            Selection::InlineFragment(inline_fragment) => {
                if let Some(field) = find(document, &inline_fragment.node.selection_set.node, name)
                {
                    return Some(field);
                }
            }
            Selection::FragmentSpread(fragment_spread) => {
                if let Some(fragment) = document
                    .fragments
                    .get(&fragment_spread.node.fragment_name.node)
                {
                    if let Some(field) = find(document, &fragment.node.selection_set.node, name) {
                        return Some(field);
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[async_std::test]
    async fn test_look_ahead() {
        #[SimpleObject(internal)]
        struct Detail {
            c: i32,
            d: i32,
        }

        #[SimpleObject(internal)]
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

        schema
            .execute(
                r#"{
            obj(n: 1) {
                a
            }
        }"#,
            )
            .await
            .unwrap();

        schema
            .execute(
                r#"{
            obj(n: 1) {
                k:a
            }
        }"#,
            )
            .await
            .unwrap();

        schema
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
            .unwrap();

        schema
            .execute(
                r#"{
            obj(n: 3) {
                b
            }
        }"#,
            )
            .await
            .unwrap();

        schema
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
            .unwrap();

        schema
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
            .unwrap();

        schema
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
            .unwrap();

        schema
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
            .unwrap();
    }
}
