use async_graphql::*;

#[tokio::test]
pub async fn test_input_object_default_value() {
    #[derive(InputObject)]
    struct MyInput {
        #[graphql(default = 999)]
        a: i32,

        #[graphql(default_with = "vec![1, 2, 3]")]
        b: Vec<i32>,

        #[graphql(default = "abc")]
        c: String,

        #[graphql(default = 999)]
        d: i32,

        #[graphql(default = 999)]
        e: i32,
    }

    struct MyOutput {
        a: i32,
        b: Vec<i32>,
        c: String,
        d: Option<i32>,
        e: Option<i32>,
    }

    #[Object]
    impl MyOutput {
        async fn a(&self) -> i32 {
            self.a
        }

        async fn b(&self) -> &Vec<i32> {
            &self.b
        }

        async fn c(&self) -> &String {
            &self.c
        }

        async fn d(&self) -> &Option<i32> {
            &self.d
        }

        async fn e(&self) -> &Option<i32> {
            &self.e
        }
    }

    struct Root;

    #[Object]
    impl Root {
        async fn a(&self, input: MyInput) -> MyOutput {
            MyOutput {
                a: input.a,
                b: input.b,
                c: input.c,
                d: Some(input.d),
                e: Some(input.e),
            }
        }
    }

    let schema = Schema::new(Root, EmptyMutation, EmptySubscription);
    let query = r#"{
            a(input:{e:777}) {
                a b c d e
            }
        }"#
    .to_owned();
    assert_eq!(
        schema.execute(&query).await.data,
        value!({
            "a": {
                "a": 999,
                "b": [1, 2, 3],
                "c": "abc",
                "d": 999,
                "e": 777,
            }
        })
    );
}

#[tokio::test]
pub async fn test_inputobject_derive_and_item_attributes() {
    use serde::Deserialize;

    #[derive(Deserialize, PartialEq, Debug, InputObject)]
    struct MyInputObject {
        #[serde(alias = "other")]
        real: i32,
    }

    assert_eq!(
        serde_json::from_str::<MyInputObject>(r#"{ "other" : 100 }"#).unwrap(),
        MyInputObject { real: 100 }
    );
}

#[tokio::test]
pub async fn test_inputobject_flatten_recursive() {
    #[derive(InputObject, Debug, Eq, PartialEq)]
    struct A {
        a: i32,
    }

    #[derive(InputObject, Debug, Eq, PartialEq)]
    struct B {
        #[graphql(default = 70)]
        b: i32,
        #[graphql(flatten)]
        a_obj: A,
    }

    #[derive(InputObject, Debug, Eq, PartialEq)]
    struct MyInputObject {
        #[graphql(flatten)]
        b_obj: B,
        c: i32,
    }

    assert_eq!(
        MyInputObject::parse(Some(value!({
           "a": 10,
           "b": 20,
           "c": 30,
        })))
        .unwrap(),
        MyInputObject {
            b_obj: B {
                b: 20,
                a_obj: A { a: 10 }
            },
            c: 30,
        }
    );

    assert_eq!(
        MyInputObject {
            b_obj: B {
                b: 20,
                a_obj: A { a: 10 }
            },
            c: 30,
        }
        .to_value(),
        value!({
           "a": 10,
           "b": 20,
           "c": 30,
        })
    );

    struct Query;

    #[Object]
    impl Query {
        async fn test(&self, input: MyInputObject) -> i32 {
            input.c + input.b_obj.b + input.b_obj.a_obj.a
        }

        async fn test_with_default(
            &self,
            #[graphql(default_with = r#"MyInputObject {
            b_obj: B {
                b: 2,
                a_obj: A { a: 1 }
            },
            c: 3,
        }"#)]
            input: MyInputObject,
        ) -> i32 {
            input.c + input.b_obj.b + input.b_obj.a_obj.a
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute(
                r#"{
            test(input:{a:10, b: 20, c: 30})
        }"#
            )
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "test": 60,
        })
    );

    assert_eq!(
        schema
            .execute(
                r#"{
            test(input:{a:10, c: 30})
        }"#
            )
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "test": 110,
        })
    );

    assert_eq!(
        schema
            .execute(
                r#"{
            testWithDefault
        }"#
            )
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "testWithDefault": 6,
        })
    );
}

#[tokio::test]
pub async fn test_inputobject_flatten_multiple() {
    #[derive(InputObject, Debug, Eq, PartialEq)]
    struct A {
        a: i32,
    }

    #[derive(InputObject, Debug, Eq, PartialEq)]
    struct B {
        b: i32,
    }

    #[derive(InputObject, Debug, Eq, PartialEq)]
    struct C {
        c: i32,
    }

    #[derive(InputObject, Debug, Eq, PartialEq)]
    struct Abc {
        #[graphql(flatten)]
        a: A,

        #[graphql(flatten)]
        b: B,

        #[graphql(flatten)]
        c: C,
    }

    assert_eq!(
        Abc::parse(Some(value!({
           "a": 10,
           "b": 20,
           "c": 30,
        })))
        .unwrap(),
        Abc {
            a: A { a: 10 },
            b: B { b: 20 },
            c: C { c: 30 }
        }
    );

    assert_eq!(
        Abc {
            a: A { a: 10 },
            b: B { b: 20 },
            c: C { c: 30 }
        }
        .to_value(),
        value!({
           "a": 10,
           "b": 20,
           "c": 30,
        })
    );
}

#[tokio::test]
pub async fn test_input_object_skip_field() {
    #[derive(InputObject)]
    struct MyInput2 {
        a: i32,
        #[graphql(skip)]
        b: i32,
    }

    struct Root;

    #[Object]
    impl Root {
        async fn a(&self, input: MyInput2) -> i32 {
            assert_eq!(input.b, i32::default());
            input.a
        }
    }

    let schema = Schema::new(Root, EmptyMutation, EmptySubscription);
    let query = r#"{
            a(input:{a: 777})
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "a": 777
        })
    );
}

#[tokio::test]
pub async fn test_box_input_object() {
    #[derive(InputObject)]
    struct MyInput {
        value: i32,
        input: Option<Box<MyInput>>,
    }

    struct Root;

    #[Object]
    impl Root {
        async fn q(&self, input: MyInput) -> i32 {
            input.value
                + input.input.as_ref().unwrap().value
                + input.input.as_ref().unwrap().input.as_ref().unwrap().value
        }
    }

    let schema = Schema::new(Root, EmptyMutation, EmptySubscription);
    let query = r#"{
            q(input: {value: 100, input: { value: 200, input: { value: 300 } } })
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "q": 600
        })
    );
}

#[tokio::test]
pub async fn test_both_input_output() {
    #[derive(SimpleObject, InputObject)]
    #[graphql(input_name = "MyObjectInput")]
    #[allow(dead_code)]
    struct MyObject {
        #[graphql(default = 10)]
        a: i32,
        b: bool,
        #[graphql(skip)]
        c: String,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self, input: MyObject) -> MyObject {
            input
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ obj(input: {a: 1, b: true}) { a b } }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "obj": {
                "a": 1,
                "b": true,
            }
        })
    );

    assert_eq!(
        schema
            .execute("{ obj(input: {b: true}) { a b } }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "obj": {
                "a": 10,
                "b": true,
            }
        })
    );

    assert_eq!(<MyObject as InputType>::type_name(), "MyObjectInput");
    assert_eq!(<MyObject as OutputTypeMarker>::type_name(), "MyObject");
}

#[tokio::test]
pub async fn test_both_input_output_generic() {
    #[derive(SimpleObject, InputObject)]
    #[graphql(concrete(name = "MyObjectU32", params(u32)))]
    #[graphql(concrete(name = "MyObjectString", params(String)))]
    #[allow(dead_code)]
    struct MyObject<T> where T: InputType + OutputType + OutputTypeMarker, MyObject<T>: async_graphql::OutputTypeMarker {
        a: T,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self, input: MyObject<u32>) -> MyObject<String> {
            MyObject::<String> {
                a: format!("{}", input.a),
            }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ obj(input: {a: 123}) { a } }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "obj": {
                "a": "123",
            }
        })
    );

    assert_eq!(<MyObject<u32> as InputType>::type_name(), "MyObjectU32");
    assert_eq!(<MyObject<u32> as OutputTypeMarker>::type_name(), "MyObjectU32");
    assert_eq!(
        <MyObject<String> as InputType>::type_name(),
        "MyObjectString"
    );
    assert_eq!(
        <MyObject<String> as OutputTypeMarker>::type_name(),
        "MyObjectString"
    );
}

#[tokio::test]
pub async fn test_both_input_output_generic_with_nesting() {
    #[derive(Clone, Copy, PartialEq, Eq, Enum, serde::Serialize)]
    enum MyEnum {
        Option1,
        Option2,
    }

    #[derive(SimpleObject, InputObject)]
    #[graphql(concrete(name = "MyObjectU32", params(u32)))]
    #[graphql(concrete(
        name = "MyObjectMyEnum",
        input_name = "MyObjectMyEnumInput",
        params(MyEnum)
    ))]
    #[allow(dead_code)]
    struct MyObject<T> where T: InputType + OutputType + OutputTypeMarker, MyObject<T>: async_graphql::OutputTypeMarker {
        a: T,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self, input: MyObject<MyEnum>) -> MyObject<MyEnum> {
            MyObject::<MyEnum> { a: input.a }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ obj(input: {a: OPTION_1}) { a } }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "obj": {
                "a": "OPTION_1",
            }
        })
    );

    assert_eq!(<MyObject<u32> as InputType>::type_name(), "MyObjectU32");
    assert_eq!(<MyObject<u32> as OutputTypeMarker>::type_name(), "MyObjectU32");
    assert_eq!(
        <MyObject<MyEnum> as InputType>::type_name(),
        "MyObjectMyEnumInput"
    );
    assert_eq!(
        <MyObject<MyEnum> as OutputTypeMarker>::type_name(),
        "MyObjectMyEnum"
    );
}

#[tokio::test]
pub async fn test_both_input_output_2() {
    #[derive(SimpleObject, InputObject)]
    #[graphql(name = "MyObj", input_name = "MyObjectInput")]
    #[allow(dead_code)]
    struct MyObject {
        #[graphql(default = 10)]
        a: i32,
        b: bool,
        #[graphql(skip)]
        c: String,
    }

    assert_eq!(<MyObject as InputType>::type_name(), "MyObjectInput");
    assert_eq!(<MyObject as OutputTypeMarker>::type_name(), "MyObj");
}

#[test]
#[should_panic]
pub fn test_both_input_output_with_same_name() {
    #[derive(SimpleObject, InputObject)]
    #[allow(dead_code)]
    struct MyObject {
        #[graphql(default = 10)]
        a: i32,
        b: bool,
        #[graphql(skip)]
        c: String,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self, input: MyObject) -> MyObject {
            input
        }
    }

    Schema::new(Query, EmptyMutation, EmptySubscription);
}

#[tokio::test]
pub async fn test_both_input_output_flatten() {
    #[derive(SimpleObject, InputObject)]
    #[graphql(input_name = "ABCInput")]
    #[graphql(name = "ABC")]
    #[allow(clippy::upper_case_acronyms)]
    struct ABC {
        a: i32,
        #[graphql(flatten)]
        bc: BC,
    }

    #[derive(SimpleObject, InputObject)]
    #[graphql(input_name = "BCInput")]
    struct BC {
        b: i32,
        c: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self, input: ABC) -> ABC {
            input
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ obj(input: { a: 1, b: 2, c: 3 }) { a b c } }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "obj": {
                "a": 1,
                "b": 2,
                "c": 3
            }
        })
    );
}

#[tokio::test]
pub async fn test_skip_input() {
    #[derive(SimpleObject, InputObject)]
    #[graphql(input_name = "MyObjectInput")]
    #[allow(dead_code)]
    struct MyObject {
        a: i32,
        #[graphql(skip_input)]
        b: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self, input: MyObject) -> MyObject {
            input
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ obj(input: { a: 1 }) { a b } }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "obj": {
                "a": 1,
                "b": 0,
            }
        })
    );
}

#[tokio::test]
pub async fn test_skip_output() {
    #[derive(SimpleObject, InputObject)]
    #[graphql(input_name = "MyObjectInput")]
    #[allow(dead_code)]
    struct MyObject {
        a: i32,
        #[graphql(skip_output)]
        b: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self, input: MyObject) -> MyObject {
            input
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ obj(input: { a: 1, b: 2 }) { a } }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "obj": {
                "a": 1,
            }
        })
    );
}

#[tokio::test]
pub async fn test_complex_output() {
    #[derive(SimpleObject, InputObject)]
    #[graphql(input_name = "MyObjectInput")]
    #[graphql(complex)]
    #[allow(dead_code)]
    struct MyObject {
        a: i32,
    }

    #[ComplexObject]
    impl MyObject {
        async fn double(&self) -> i32 {
            self.a * 2
        }
    }

    struct Query;
    #[Object]
    impl Query {
        async fn obj(&self, input: MyObject) -> MyObject {
            input
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ obj(input: { a: 1 }) { a, double } }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "obj": {
                "a": 1,
                "double": 2,
            }
        })
    );
}

#[tokio::test]
pub async fn test_input_object_process_with() {
    mod processor {
        pub fn string(input: &mut String) {
            while let Some(ch) = input.pop() {
                if !ch.is_whitespace() {
                    input.push(ch);
                    break;
                }
            }
        }
    }
    #[derive(InputObject)]
    struct MyInput {
        // processor does nothing on default value
        #[graphql(default = "  ", process_with = "processor::string")]
        a: String,

        #[graphql(process_with = "processor::string")]
        b: String,
    }

    struct MyOutput {
        a: String,
        b: String,
    }

    #[Object]
    impl MyOutput {
        async fn a(&self) -> &String {
            &self.a
        }

        async fn b(&self) -> &String {
            &self.b
        }
    }

    struct Root;

    #[Object]
    impl Root {
        async fn a(&self, input: MyInput) -> MyOutput {
            MyOutput {
                a: input.a,
                b: input.b,
            }
        }
    }

    let schema = Schema::new(Root, EmptyMutation, EmptySubscription);
    let query = r#"{
            a(input:{b: "test b   "}) {
                a b
            }
        }"#
    .to_owned();
    assert_eq!(
        schema.execute(&query).await.data,
        value!({
            "a": {
                "a": "  ",
                "b": "test b",
            }
        })
    );

    let schema = Schema::new(Root, EmptyMutation, EmptySubscription);
    let query = r#"{
            a(input:{a: "test a ", b: "test"}) {
                a b
            }
        }"#
    .to_owned();
    assert_eq!(
        schema.execute(&query).await.data,
        value!({
            "a": {
                "a": "test a",
                "b": "test",
            }
        })
    );
}

#[tokio::test]
pub async fn test_input_object_validator() {
    fn check_my_object(obj: &MyInput) -> Result<(), &'static str> {
        if obj.a < 100 || obj.b < 100 {
            Err("invalid MyInput")
        } else {
            Ok(())
        }
    }

    #[derive(InputObject)]
    #[graphql(validator = "check_my_object")]
    struct MyInput {
        a: i32,
        b: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn a(&self, input: MyInput) -> i32 {
            input.a + input.b
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert_eq!(
        schema
            .execute("{ a(input: { a: 200, b: 300 }) }")
            .await
            .data,
        value!({ "a": 500 })
    );

    assert_eq!(
        schema
            .execute("{ a(input: { a: 100, b: 25 }) }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "MyInput": invalid MyInput"#.to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 12
            }],
            path: vec![PathSegment::Field("a".to_string())],
            extensions: None
        }]
    );
}

#[tokio::test]
pub async fn test_custom_validator_with_extensions_input() {
    struct MyValidator {
        expect: i32,
    }

    impl MyValidator {
        pub fn new(n: i32) -> Self {
            MyValidator { expect: n }
        }
    }

    impl CustomValidator<i32> for MyValidator {
        fn check(&self, value: &i32) -> Result<(), InputValueError<i32>> {
            if *value == self.expect {
                Ok(())
            } else {
                Err(
                    InputValueError::custom(format!("expect 100, actual {}", value))
                        .with_extension("code", 99),
                )
            }
        }
    }

    #[derive(InputObject, Debug)]
    struct ValueInput {
        #[graphql(validator(custom = "MyValidator::new(100)"))]
        v: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self, n: ValueInput) -> i32 {
            n.v
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ value(n: {v: 100}) }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "value": 100 })
    );

    let mut error_extensions = ErrorExtensionValues::default();
    error_extensions.set("code", 99);
    assert_eq!(
        schema
            .execute("{ value(n: {v: 11}) }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "Int": expect 100, actual 11 (occurred while parsing "ValueInput")"#.to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 12
            }],
            path: vec![PathSegment::Field("value".to_string())],
            extensions: Some(error_extensions)
        }]
    );
}
