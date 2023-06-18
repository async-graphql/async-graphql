#![allow(clippy::uninlined_format_args)]

use async_graphql::*;

#[tokio::test]
pub async fn test_derived_field_object() {
    use serde::{Deserialize, Serialize};

    struct Query;

    #[derive(Serialize, Deserialize)]
    struct ValueDerived(String);

    scalar!(ValueDerived);

    impl From<i32> for ValueDerived {
        fn from(value: i32) -> Self {
            ValueDerived(format!("{}", value))
        }
    }

    #[Object]
    impl Query {
        #[graphql(derived(name = "value2", into = "ValueDerived"))]
        async fn value1(&self, #[graphql(default = 100)] input: i32) -> i32 {
            input
        }
    }

    let query = "{ value1 value2 }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "value1": 100,
            "value2": "100",
        })
    );

    let query = "{ value1(input: 1) value2(input: 2) }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "value1": 1,
            "value2": "2",
        })
    );
}

#[tokio::test]
pub async fn test_derived_field_object_with() {
    use serde::{Deserialize, Serialize};

    struct Query;

    #[derive(Serialize, Deserialize)]
    struct ValueDerived(String);

    scalar!(ValueDerived);

    impl From<i32> for ValueDerived {
        fn from(value: i32) -> Self {
            ValueDerived(format!("{}", value))
        }
    }

    fn option_to_option<T, U: From<T>>(value: Option<T>) -> Option<U> {
        value.map(|x| x.into())
    }

    #[Object]
    impl Query {
        #[graphql(derived(
            name = "value2",
            into = "Option<ValueDerived>",
            with = "option_to_option"
        ))]
        async fn value1(&self, #[graphql(default = 100)] input: i32) -> Option<i32> {
            Some(input)
        }
    }

    let query = "{ value1 value2 }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "value1": 100,
            "value2": "100",
        })
    );

    let query = "{ value1(input: 1) value2(input: 2) }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "value1": 1,
            "value2": "2",
        })
    );
}

#[tokio::test]
pub async fn test_derived_field_simple_object() {
    use serde::{Deserialize, Serialize};

    struct Query;

    #[derive(Serialize, Deserialize)]
    struct ValueDerived(String);

    scalar!(ValueDerived);

    impl From<i32> for ValueDerived {
        fn from(value: i32) -> Self {
            ValueDerived(format!("{}", value))
        }
    }

    #[derive(SimpleObject)]
    struct TestObj {
        #[graphql(owned, derived(name = "value2", into = "ValueDerived"))]
        pub value1: i32,
    }

    #[Object]
    impl Query {
        async fn test(&self, #[graphql(default = 100)] input: i32) -> TestObj {
            TestObj { value1: input }
        }
    }

    let query = "{ test { value1 value2 } }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "test": {
                "value1": 100,
                "value2": "100",
            }
        })
    );

    let query = "{ test(input: 2) { value1 value2 }}";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    dbg!(schema.execute(query).await);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "test": {
                "value1": 2,
                "value2": "2",
            }
        })
    );
}

#[tokio::test]
pub async fn test_derived_field_simple_object_option() {
    use serde::{Deserialize, Serialize};

    struct Query;

    #[derive(Serialize, Deserialize, Clone)]
    struct ValueDerived(String);

    #[derive(Serialize, Deserialize, Clone)]
    struct ValueDerived2(String);

    scalar!(ValueDerived);
    scalar!(ValueDerived2);

    impl From<ValueDerived> for ValueDerived2 {
        fn from(value: ValueDerived) -> Self {
            ValueDerived2(value.0)
        }
    }

    fn option_to_option<T, U: From<T>>(value: Option<T>) -> Option<U> {
        value.map(|x| x.into())
    }

    fn vec_to_vec<T, U: From<T>>(value: Vec<T>) -> Vec<U> {
        value.into_iter().map(|x| x.into()).collect()
    }

    fn vecopt_to_vecopt<T, U: From<T>>(value: Vec<Option<T>>) -> Vec<Option<U>> {
        value.into_iter().map(|x| x.map(|opt| opt.into())).collect()
    }

    fn optvec_to_optvec<T, U: From<T>>(value: Option<Vec<T>>) -> Option<Vec<U>> {
        value.map(|x| x.into_iter().map(|y| y.into()).collect())
    }

    #[derive(SimpleObject)]
    struct TestObj {
        #[graphql(derived(
            owned,
            name = "value2",
            into = "Option<ValueDerived2>",
            with = "option_to_option"
        ))]
        pub value1: Option<ValueDerived>,
        #[graphql(derived(
            owned,
            name = "value_vec_2",
            into = "Vec<ValueDerived2>",
            with = "vec_to_vec"
        ))]
        pub value_vec_1: Vec<ValueDerived>,
        #[graphql(derived(
            owned,
            name = "value_opt_vec_2",
            into = "Option<Vec<ValueDerived2>>",
            with = "optvec_to_optvec"
        ))]
        pub value_opt_vec_1: Option<Vec<ValueDerived>>,
        #[graphql(derived(
            owned,
            name = "value_vec_opt_2",
            into = "Vec<Option<ValueDerived2>>",
            with = "vecopt_to_vecopt"
        ))]
        pub value_vec_opt_1: Vec<Option<ValueDerived>>,
    }

    #[Object]
    impl Query {
        async fn test(&self) -> TestObj {
            TestObj {
                value1: Some(ValueDerived("Test".to_string())),
                value_vec_1: vec![ValueDerived("Test".to_string())],
                value_opt_vec_1: Some(vec![ValueDerived("Test".to_string())]),
                value_vec_opt_1: vec![Some(ValueDerived("Test".to_string()))],
            }
        }
    }

    let query = "{ test { value1 value2 valueVec1 valueVec2 valueOptVec1 valueOptVec2 } }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "test": {
                "value1": "Test",
                "value2": "Test",
                "valueVec1": vec!["Test"],
                "valueVec2": vec!["Test"],
                "valueOptVec1": vec!["Test"],
                "valueOptVec2": vec!["Test"],
            }
        })
    );
}

#[tokio::test]
pub async fn test_derived_field_complex_object() {
    use serde::{Deserialize, Serialize};

    #[derive(SimpleObject)]
    #[graphql(complex)]
    struct MyObj {
        a: i32,
        #[graphql(owned, derived(name = "f", into = "ValueDerived"))]
        b: i32,
    }

    #[derive(Serialize, Deserialize)]
    struct ValueDerived(String);

    scalar!(ValueDerived);

    impl From<i32> for ValueDerived {
        fn from(value: i32) -> Self {
            ValueDerived(format!("{}", value))
        }
    }

    #[ComplexObject]
    impl MyObj {
        async fn c(&self) -> i32 {
            self.a + self.b
        }

        #[graphql(derived(name = "e", into = "ValueDerived"))]
        async fn d(&self, v: i32) -> i32 {
            self.a + self.b + v
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyObj {
            MyObj { a: 10, b: 20 }
        }
    }

    let query = "{ obj { a b c d(v:100) e(v: 200) f } }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    dbg!(schema.execute(query).await);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "obj": {
                "a": 10,
                "b": 20,
                "c": 30,
                "d": 130,
                "e": "230",
                "f": "20",
            },
        })
    );
}

#[tokio::test]
pub async fn test_derived_field_complex_object_derived() {
    use serde::{Deserialize, Serialize};

    #[derive(SimpleObject)]
    #[graphql(complex)]
    struct MyObj {
        a: i32,
        #[graphql(owned, derived(name = "f", into = "ValueDerived"))]
        b: i32,
    }

    #[derive(Serialize, Deserialize)]
    struct ValueDerived(String);

    scalar!(ValueDerived);

    impl From<i32> for ValueDerived {
        fn from(value: i32) -> Self {
            ValueDerived(format!("{}", value))
        }
    }

    fn option_to_option<T, U: From<T>>(value: Option<T>) -> Option<U> {
        value.map(|x| x.into())
    }

    #[ComplexObject]
    impl MyObj {
        async fn c(&self) -> i32 {
            self.a + self.b
        }

        #[graphql(derived(name = "e", into = "Option<ValueDerived>", with = "option_to_option"))]
        async fn d(&self, v: i32) -> Option<i32> {
            Some(self.a + self.b + v)
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyObj {
            MyObj { a: 10, b: 20 }
        }
    }

    let query = "{ obj { a b c d(v:100) e(v: 200) f } }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "obj": {
                "a": 10,
                "b": 20,
                "c": 30,
                "d": 130,
                "e": "230",
                "f": "20",
            },
        })
    );
}
