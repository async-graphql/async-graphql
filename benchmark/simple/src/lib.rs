use async_graphql::*;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn value_i32(&self) -> i32 {
        999
    }

    async fn obj(&self) -> MyObj {
        MyObj
    }
}

pub struct MyObj;

#[Object]
impl MyObj {
    async fn value_i32(&self) -> i32 {
        999
    }

    async fn value_list(&self) -> &[i32] {
        &[1, 2, 3, 4, 5, 6, 7, 8, 9]
    }

    async fn obj(&self) -> MyObj {
        MyObj
    }
}

lazy_static::lazy_static! {
    pub static ref S: Schema<QueryRoot, EmptyMutation, EmptySubscription> = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
    // static ref D: Document = parse_query(Q).unwrap();
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
