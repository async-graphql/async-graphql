use async_graphql::*;
use async_graphql_parser::{parse_query, query::Document};
use async_std::task;
pub use http::GQLResponse;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    #[field]
    async fn value_i32(&self) -> i32 {
        999
    }

    #[field]
    async fn obj(&self) -> MyObj {
        MyObj
    }
}

pub struct MyObj;

#[Object]
impl MyObj {
    #[field]
    async fn value_i32(&self) -> i32 {
        999
    }

    #[field]
    async fn value_list(&self) -> &[i32] {
        &[1, 2, 3, 4, 5, 6, 7, 8, 9]
    }

    #[field]
    async fn obj(&self) -> MyObj {
        MyObj
    }
}

pub const Q: &str = r#"{
    valueI32 obj {
        valueI32 valueList obj {
            valueI32 valueList obj {
                valueI32 valueList obj {
                    valueI32 valueList obj {
                        valueI32 valueList obj {
                            valueI32 valueList obj {
                                valueI32 valueList obj {
                                    valueI32 valueList obj {
                                        valueI32 valueList obj {
                                            valueI32 valueList obj {
                                                valueI32 valueList obj {
                                                    valueI32 valueList obj {
                                                        valueI32 valueList obj {
                                                            valueI32 valueList obj {
                                                                valueI32 valueList
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}"#;

lazy_static::lazy_static! {
    static ref S: Schema<QueryRoot, EmptyMutation, EmptySubscription> = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    // static ref D: Document = parse_query(Q).unwrap();
}

pub fn run(q: &str) -> Result<QueryResponse> {
    task::block_on(async { S.execute(q).await })
}

pub fn parse(q: &str) -> Document {
    parse_query(q).unwrap()
}

// pub fn validate() {
//     check_rules(&S.env.registry, &D, S.validation_mode).unwrap();
// }
//
// pub fn resolve() {
//     do_resolve(...).unwrap();
// }

pub fn serialize(r: &GQLResponse) -> String {
    serde_json::to_string(&r).unwrap()
}
