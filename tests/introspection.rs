use async_graphql::*;
use async_std::stream;
use async_std::stream::Stream;

#[derive(Clone, Debug)]
struct Circle {
    radius: f32,
}

#[Object(desc = "Circle")]
impl Circle {
    async fn scale(&self, s: f32) -> TestInterface {
        Circle {
            radius: self.radius * s,
        }
        .into()
    }
}

#[derive(Clone, Debug)]
struct Square {
    width: f32,
}

#[Object(desc = "Square")]
impl Square {
    #[field(deprecation = "Field scale is deprecated")]
    async fn scale(&self, s: f32) -> TestInterface {
        Square {
            width: self.width * s,
        }
        .into()
    }
}

#[Interface(field(name = "scale", type = "TestInterface", arg(name = "s", type = "f32")))]
#[derive(Clone, Debug)]
enum TestInterface {
    Circle(Circle),
    Square(Square),
}

#[Union(desc = "Test Union")]
#[derive(Clone, Debug)]
enum TestUnion {
    Circle(Circle),
    Square(Square),
}

#[Enum(desc = "Test Enum")]
enum TestEnum {
    #[item(desc = "Kind 1")]
    Kind1,

    #[item(desc = "Kind 2", deprecation = "Kind 2 deprecated")]
    Kind2,
}

#[SimpleObject]
#[derive(Clone, Debug)]
struct SimpleList {
    items: Vec<String>,
}

#[SimpleObject]
#[derive(Clone, Debug)]
struct SimpleOption {
    required: i32,
    optional: Option<i32>,
}

/// TestScalar
#[derive(Clone, Debug)]
struct TestScalar(i32);

#[Scalar(desc = "Test scalar")]
impl ScalarType for TestScalar {
    fn parse(_value: Value) -> InputValueResult<Self> {
        Ok(TestScalar(42))
    }

    fn is_valid(_value: &Value) -> bool {
        true
    }

    fn to_value(&self) -> Value {
        Value::Number(self.0.into())
    }
}

/// Is SimpleObject
/// and some more ```lorem ipsum```
#[SimpleObject]
struct SimpleObject {
    /// Value a with # 'some' `markdown`"."
    /// and some more lorem ipsum
    a: i32,

    #[field(desc = "Value b description")]
    b: String,

    /// Value c doc comment
    #[field(desc = "Value c description")]
    c: ID,

    #[field(desc = "")]
    d: SimpleOption,

    #[field(deprecation = "Field e is deprecated")]
    e: bool,

    f: TestEnum,

    g: TestInterface,

    h: TestUnion,

    i: SimpleList,

    j: TestScalar,
}

struct Query;

#[Object(desc = "Global query")]
impl Query {
    /// Get a simple object
    async fn simple_object(&self) -> SimpleObject {
        unimplemented!()
    }
}

#[async_graphql::InputObject(desc = "Simple Input")]
pub struct SimpleInput {
    pub a: String,
}

struct Mutation;

#[Object(desc = "Global mutation")]
impl Mutation {
    /// simple_mutation description
    /// line2
    /// line3
    async fn simple_mutation(&self, _input: SimpleInput) -> SimpleObject {
        unimplemented!()
    }
}

struct Subscription;

#[Subscription(desc = "Global subscription")]
impl Subscription {
    /// simple_subscription description
    async fn simple_subscription(&self, #[arg(default = 1)] step: i32) -> impl Stream<Item = i32> {
        stream::once(step)
    }
}

// #[async_std::test]
// pub async fn test_introspection_schema() {
//     let schema = Schema::new(Query, Mutation, Subscription);

//     let query = r#"
//     {
//         __schema {
//             directives {
//               name
//               locations
//             }
//             subscriptionType {
//               name
//               fields { name }
//             }
//             types {
//               name
//             }
//             queryType {
//               name
//             }
//             mutationType {
//               name
//             }
//             queryType {
//               name
//             }
//         }
//    }
//    "#;

//     let res_json = serde_json::json!({
//         "__schema": {
//         }
//     });

//     let res = schema.execute(query).await.unwrap().data;

//     // pretty print result
//     // println!("{}", serde_json::to_string_pretty(&res).unwrap());

//     assert_eq!(res, res_json)
// }

// #[async_std::test]
// pub async fn test_introspection_documentation() {
//     let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
//
//     let query = r#"
//     {
//         __type(name: "SimpleObject") {
//             name
//             description
//             fields {
//                 name
//                 description
//             }
//         }
//    }
//    "#;
//
//     let res_json = serde_json::json!({
//         "__type": {
//             "name": "SimpleObject",
//             "description": "Is SimpleObject",
//             "fields": [
//               {
//                 "name": "a",
//                 "description": "Value a with # 'some' `markdown`\".\""
//               },
//               {
//                 "name": "b",
//                 "description": "Value b description"
//               },
//               {
//                 "name": "c",
//                 "description": "Value c description"
//               },
//               {
//                 "name": "d",
//                 "description": ""
//               },
//               {
//                 "name": "f",
//                 "description": null
//               },
//               {
//                 "name": "g",
//                 "description": null
//               },
//               {
//                 "name": "h",
//                 "description": null
//               },
//               {
//                 "name": "i",
//                 "description": null
//               },
//               {
//                 "name": "j",
//                 "description": null
//               }
//             ]
//         }
//     });
//
//     let res = schema.execute(query).await.unwrap().data;
//
//     assert_eq!(res, res_json)
// }

#[async_std::test]
pub async fn test_introspection_deprecation() {
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    let get_object_query = |obj, is_deprecated| {
        format!(
            r#"
        {{
            __type(name: "{}") {{
                fields(includeDeprecated: {})  {{
                    name
                    isDeprecated
                    deprecationReason
                }}
            }}
       }}
       "#,
            obj, is_deprecated
        )
    };

    // SimpleObject with deprecated inclusive
    let mut query = get_object_query("SimpleObject", "true");

    let mut res_json = serde_json::json!({
        "__type": {
            "fields": [
              {
                "name": "a",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "b",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "c",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "d",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "e",
                "isDeprecated": true,
                "deprecationReason": "Field e is deprecated"
              },
              {
                "name": "f",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "g",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "h",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "i",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "j",
                "isDeprecated": false,
                "deprecationReason": null
              }
            ]
        }
    });

    let mut res = schema.execute(&query).await.unwrap().data;

    assert_eq!(res, res_json);

    // SimpleObject with deprecated fields exclusive
    query = get_object_query("SimpleObject", "false");

    res_json = serde_json::json!({
        "__type": {
            "fields": [
              {
                "name": "a",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "b",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "c",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "d",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "f",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "g",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "h",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "i",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "j",
                "isDeprecated": false,
                "deprecationReason": null
              }
            ]
        }
    });

    res = schema.execute(&query).await.unwrap().data;

    assert_eq!(res, res_json);

    // Object with only one deprecated field inclusive
    query = get_object_query("Square", "true");

    res_json = serde_json::json!({
        "__type": {
            "fields": [
                {
                "name": "scale",
                "isDeprecated": true,
                "deprecationReason": "Field scale is deprecated"
                }
            ]
        }
    });

    res = schema.execute(&query).await.unwrap().data;

    assert_eq!(res, res_json);

    // Object with only one deprecated field exclusive
    query = get_object_query("Square", "false");

    res_json = serde_json::json!({
        "__type": {
            "fields": []
        }
    });

    res = schema.execute(&query).await.unwrap().data;

    assert_eq!(res, res_json);

    let get_enum_query = |obj, is_deprecated| {
        format!(
            r#"
        {{
            __type(name: "{}") {{
                enumValues(includeDeprecated: {})  {{
                    name
                    isDeprecated
                    deprecationReason
                }}
            }}
       }}
       "#,
            obj, is_deprecated
        )
    };

    // Enum with deprecated value inclusive
    query = get_enum_query("TestEnum", "true");

    res_json = serde_json::json!({
        "__type": {
            "enumValues": [
              {
                "name": "KIND_1",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "KIND_2",
                "isDeprecated": true,
                "deprecationReason": "Kind 2 deprecated"
              }
            ]
        }
    });

    res = schema.execute(&query).await.unwrap().data;

    assert_eq!(res, res_json);

    // Enum with deprecated value exclusive
    query = get_enum_query("TestEnum", "false");

    res_json = serde_json::json!({
        "__type": {
            "enumValues": [
              {
                "name": "KIND_1",
                "isDeprecated": false,
                "deprecationReason": null
              }
            ]
        }
    });

    res = schema.execute(&query).await.unwrap().data;

    assert_eq!(res, res_json);
}

#[async_std::test]
pub async fn test_introspection_type_kind() {
    let schema = Schema::new(Query, Mutation, EmptySubscription);

    let get_type_kind_query = |obj| {
        format!(
            r#"{{
              __type(name: "{}") {{
                  name
                  kind
              }}
          }}"#,
            obj
        )
    };

    // Test simple object
    let mut query = get_type_kind_query("SimpleObject");

    let mut res_json = serde_json::json!({
        "__type": {
            "name": "SimpleObject",
            "kind": "OBJECT"
        }
    });

    let mut res = schema.execute(&query).await.unwrap().data;

    assert_eq!(res, res_json);

    // Test object
    query = get_type_kind_query("Square");

    res_json = serde_json::json!({
        "__type": {
            "name": "Square",
            "kind": "OBJECT"
        }
    });

    res = schema.execute(&query).await.unwrap().data;

    assert_eq!(res, res_json);

    // Test enum
    query = get_type_kind_query("TestEnum");

    res_json = serde_json::json!({
        "__type": {
            "name": "TestEnum",
            "kind": "ENUM"
        }
    });

    res = schema.execute(&query).await.unwrap().data;

    assert_eq!(res, res_json);

    // Test union
    query = get_type_kind_query("TestUnion");

    res_json = serde_json::json!({
        "__type": {
            "name": "TestUnion",
            "kind": "UNION"
        }
    });

    res = schema.execute(&query).await.unwrap().data;

    assert_eq!(res, res_json);

    // Test scalar
    query = get_type_kind_query("ID");

    res_json = serde_json::json!({
        "__type": {
            "name": "ID",
            "kind": "SCALAR"
        }
    });

    res = schema.execute(&query).await.unwrap().data;

    assert_eq!(res, res_json);

    let get_field_kind_query = |obj| {
        format!(
            r#"{{
              __type(name: "{}") {{
                  fields {{
                      name
                      type {{ name kind ofType {{ name kind }} }}
                  }}
              }}
          }}"#,
            obj
        )
    };

    // Test list
    query = get_field_kind_query("SimpleList");

    res_json = serde_json::json!({
        "__type": {
            "fields": [
                {
                "name": "items",
                "type": {
                    "name": null,
                    "kind": "NON_NULL",
                    "ofType": {
                        "name": null,
                        "kind": "LIST"
                    }
                }
                }
            ]
          }
    });

    res = schema.execute(&query).await.unwrap().data;

    assert_eq!(res, res_json);

    // Test NON_NULL
    query = get_field_kind_query("SimpleOption");

    res_json = serde_json::json!({
        "__type": {
            "fields": [
              {
                "name": "required",
                "type": {
                  "name": null,
                  "kind": "NON_NULL",
                  "ofType": {
                    "name": "Int",
                    "kind": "SCALAR"
                  }
                }
              },
              {
                "name": "optional",
                "type": {
                  "name": "Int",
                  "kind": "SCALAR",
                  "ofType": null
                }
              }
            ]
        }
    });

    res = schema.execute(&query).await.unwrap().data;

    assert_eq!(res, res_json);
}

#[async_std::test]
pub async fn test_introspection_scalar() {
    let schema = Schema::new(Query, Mutation, EmptySubscription);

    let query = r#"
    {
        __type(name: "TestScalar") {
            kind
            name
            description
        }
   }
   "#;

    let res_json = serde_json::json!({
        "__type": {
            "kind": "SCALAR",
            "name": "TestScalar",
            "description": "Test scalar",
        }
    });

    let res = schema.execute(query).await.unwrap().data;

    assert_eq!(res, res_json)
}

#[async_std::test]
pub async fn test_introspection_union() {
    let schema = Schema::new(Query, Mutation, EmptySubscription);

    let query = r#"
    {
        __type(name: "TestUnion") {
            kind
            name
            description
            possibleTypes { name }
        }
   }
   "#;

    let res_json = serde_json::json!({
        "__type": {
            "kind": "UNION",
            "name": "TestUnion",
            "description": "Test Union",
            "possibleTypes": [
              {
                "name": "Circle"
              },
              {
                "name": "Square"
              }
            ]
        }
    });

    let res = schema.execute(query).await.unwrap().data;

    assert_eq!(res, res_json)
}

#[async_std::test]
pub async fn test_introspection_interface() {
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    // Test if possibleTypes and other fields are set
    let mut query = r#"
    {
        __type(name: "TestInterface") {
            kind
            name
            description
            possibleTypes { name }
            fields { name }
        }
   }
   "#;

    let mut res_json = serde_json::json!({
        "__type": {
            "kind": "INTERFACE",
            "name": "TestInterface",
            "description": null,
            "possibleTypes": [
              {
                "name": "Circle"
              },
              {
                "name": "Square"
              }
            ],
            "fields": [
              {
                "name": "scale"
              }
            ]
        }
    });

    let mut res = schema.execute(query).await.unwrap().data;

    assert_eq!(res, res_json);

    // Test if interfaces and other fields are set
    query = r#"
    {
        __type(name: "Circle") {
            kind
            name
            description
            interfaces { name }
            fields { name }
        }
   }
   "#;

    res_json = serde_json::json!({
        "__type": {
            "kind": "OBJECT",
            "name": "Circle",
            "description": "Circle",
            "interfaces": [
              {
                "name": "TestInterface"
              }
            ],
            "fields": [
              {
                "name": "scale"
              }
            ]
        }
    });

    res = schema.execute(query).await.unwrap().data;

    assert_eq!(res, res_json);
}

#[async_std::test]
pub async fn test_introspection_enum() {
    let schema = Schema::new(Query, Mutation, EmptySubscription);

    let query = r#"
    {
        __type(name: "TestEnum") {
            kind
            name
            description
            enumValues(includeDeprecated: true) {
                name
                description
                isDeprecated
                deprecationReason
            }
        }
   }
   "#;

    let res_json = serde_json::json!({
        "__type": {
            "kind": "ENUM",
            "name": "TestEnum",
            "description": "Test Enum",
            "enumValues": [
              {
                "name": "KIND_1",
                "description": "Kind 1",
                "isDeprecated": false,
                "deprecationReason": null
              },
              {
                "name": "KIND_2",
                "description": "Kind 2",
                "isDeprecated": true,
                "deprecationReason": "Kind 2 deprecated"
              }
            ]
        }
    });

    let res = schema.execute(query).await.unwrap().data;

    println!("{}", serde_json::to_string_pretty(&res).unwrap());

    assert_eq!(res, res_json)
}

#[async_std::test]
pub async fn test_introspection_input_object() {
    let schema = Schema::new(Query, Mutation, EmptySubscription);

    let query = r#"
    {
        __type(name: "SimpleInput") {
            kind
            name
            description
            inputFields { name }
        }
   }
   "#;

    let res_json = serde_json::json!({
        "__type": {
            "kind": "INPUT_OBJECT",
            "name": "SimpleInput",
            "description": "Simple Input",
            "inputFields": [
                {
                    "name": "a"
                }
            ],
        }
    });

    let res = schema.execute(query).await.unwrap().data;

    assert_eq!(res, res_json)
}

#[async_std::test]
pub async fn test_introspection_mutation() {
    let schema = Schema::new(Query, Mutation, EmptySubscription);

    let query = r#"
    {
        __type(name: "Mutation") {
            name
    		kind
    		description
            fields {
                description
                name
                type { kind name }
                args { name }
            }
        }
   }
   "#;

    let res_json = serde_json::json!({
        "__type": {
            "name": "Mutation",
            "kind": "OBJECT",
            "description": "Global mutation",
            "fields": [
              {
                "description": "simple_mutation description\nline2\nline3",
                "name": "simpleMutation",
                "type": {
                  "kind": "NON_NULL",
                  "name": null
                },
                "args": [
                  {
                    "name": "input"
                  }
                ]
              }
            ]
        }
    });

    let res = schema.execute(query).await.unwrap().data;

    assert_eq!(res, res_json)
}

#[async_std::test]
pub async fn test_introspection_subscription() {
    let schema = Schema::new(Query, EmptyMutation, Subscription);

    let query = r#"
    {
        __type(name: "Subscription") {
            name
    		kind
    		description
            fields {
                description
                name
                type { kind name }
                args { name }
            }
        }
   }
   "#;

    let res_json = serde_json::json!({
        "__type": {
            "name": "Subscription",
            "kind": "OBJECT",
            "description": "Global subscription",
            "fields": [
              {
                "description": "simple_subscription description",
                "name": "simpleSubscription",
                "type": {
                  "kind": "NON_NULL",
                  "name": null
                },
                "args": [
                  {
                    "name": "step"
                  }
                ]
              }
            ]
        }
    });

    let res = schema.execute(query).await.unwrap().data;

    assert_eq!(res, res_json)
}

// #[async_std::test]
// pub async fn test_introspection_full() {
//     let schema = Schema::new(Query, EmptyMutation, Subscription);
//
//     let query = r#"
//     {
//         __type(name: "SimpleObject") {
//             kind
//             name
//             description
//             fields(includeDeprecated: true) {
//                 name
//                 description
//                 args { name }
//                 type { name kind ofType { name kind } }
//                 isDeprecated
//                 deprecationReason
//             }
//             interfaces { name }
//             possibleTypes { name }
//             enumValues { name }
//             inputFields { name }
//             ofType { name }
//         }
//    }
//    "#;
//
//     let res_json = serde_json::json!({
//         "__type": {
//             "kind": "OBJECT",
//             "name": "SimpleObject",
//             "description": "Is SimpleObject",
//             "fields": [
//               {
//                 "name": "a",
//                 "description": "Value a with # 'some' `markdown`\".\"",
//                 "args": [],
//                 "type": {
//                   "name": null,
//                   "kind": "NON_NULL",
//                   "ofType": {
//                     "name": "Int",
//                     "kind": "SCALAR"
//                   }
//                 },
//                 "isDeprecated": false,
//                 "deprecationReason": null
//               },
//               {
//                 "name": "b",
//                 "description": "Value b description",
//                 "args": [],
//                 "type": {
//                   "name": null,
//                   "kind": "NON_NULL",
//                   "ofType": {
//                     "name": "String",
//                     "kind": "SCALAR"
//                   }
//                 },
//                 "isDeprecated": false,
//                 "deprecationReason": null
//               },
//               {
//                 "name": "c",
//                 "description": "Value c description",
//                 "args": [],
//                 "type": {
//                   "name": null,
//                   "kind": "NON_NULL",
//                   "ofType": {
//                     "name": "ID",
//                     "kind": "SCALAR"
//                   }
//                 },
//                 "isDeprecated": false,
//                 "deprecationReason": null
//               },
//               {
//                 "name": "d",
//                 "description": "",
//                 "args": [],
//                 "type": {
//                   "name": null,
//                   "kind": "NON_NULL",
//                   "ofType": {
//                     "name": "SimpleOption",
//                     "kind": "OBJECT"
//                   }
//                 },
//                 "isDeprecated": false,
//                 "deprecationReason": null
//               },
//               {
//                 "name": "e",
//                 "description": null,
//                 "args": [],
//                 "type": {
//                   "name": null,
//                   "kind": "NON_NULL",
//                   "ofType": {
//                     "name": "Boolean",
//                     "kind": "SCALAR"
//                   }
//                 },
//                 "isDeprecated": true,
//                 "deprecationReason": "Field e is deprecated"
//               },
//               {
//                 "name": "f",
//                 "description": null,
//                 "args": [],
//                 "type": {
//                   "name": null,
//                   "kind": "NON_NULL",
//                   "ofType": {
//                     "name": "TestEnum",
//                     "kind": "ENUM"
//                   }
//                 },
//                 "isDeprecated": false,
//                 "deprecationReason": null
//               },
//               {
//                 "name": "g",
//                 "description": null,
//                 "args": [],
//                 "type": {
//                   "name": null,
//                   "kind": "NON_NULL",
//                   "ofType": {
//                     "name": "TestInterface",
//                     "kind": "INTERFACE"
//                   }
//                 },
//                 "isDeprecated": false,
//                 "deprecationReason": null
//               },
//               {
//                 "name": "h",
//                 "description": null,
//                 "args": [],
//                 "type": {
//                   "name": null,
//                   "kind": "NON_NULL",
//                   "ofType": {
//                     "name": "TestUnion",
//                     "kind": "UNION"
//                   }
//                 },
//                 "isDeprecated": false,
//                 "deprecationReason": null
//               },
//               {
//                 "name": "i",
//                 "description": null,
//                 "args": [],
//                 "type": {
//                   "name": null,
//                   "kind": "NON_NULL",
//                   "ofType": {
//                     "name": "SimpleList",
//                     "kind": "OBJECT"
//                   }
//                 },
//                 "isDeprecated": false,
//                 "deprecationReason": null
//               },
//               {
//                 "name": "j",
//                 "description": null,
//                 "args": [],
//                 "type": {
//                   "name": null,
//                   "kind": "NON_NULL",
//                   "ofType": {
//                     "name": "TestScalar",
//                     "kind": "SCALAR"
//                   }
//                 },
//                 "isDeprecated": false,
//                 "deprecationReason": null
//               }
//             ],
//             "interfaces": [],
//             "possibleTypes": null,
//             "enumValues": null,
//             "inputFields": null,
//             "ofType": null
//         }
//     });
//
//     let res = schema.execute(query).await.unwrap().data;
//
//     // pretty print result
//     // println!("{}", serde_json::to_string_pretty(&res).unwrap());
//
//     assert_eq!(res, res_json)
// }
