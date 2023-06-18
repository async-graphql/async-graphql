#![allow(clippy::uninlined_format_args)]

use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashSet, LinkedList, VecDeque},
};

use async_graphql::*;

#[tokio::test]
pub async fn test_list_type() {
    #[derive(InputObject)]
    struct MyInput {
        value: Vec<i32>,
    }

    struct Root {
        value_vec: Vec<i32>,
        value_hash_set: HashSet<i32>,
        value_btree_set: BTreeSet<i32>,
        value_linked_list: LinkedList<i32>,
        value_vec_deque: VecDeque<i32>,
    }

    #[Object]
    impl Root {
        async fn value_vec(&self) -> Vec<i32> {
            self.value_vec.clone()
        }

        async fn value_slice(&self) -> &[i32] {
            &self.value_vec
        }

        async fn value_linked_list(&self) -> LinkedList<i32> {
            self.value_linked_list.clone()
        }

        async fn value_hash_set(&self) -> HashSet<i32> {
            self.value_hash_set.clone()
        }

        async fn value_btree_set(&self) -> BTreeSet<i32> {
            self.value_btree_set.clone()
        }

        async fn value_vec_deque(&self) -> VecDeque<i32> {
            self.value_vec_deque.clone()
        }

        async fn value_input_slice(&self, a: Vec<i32>) -> Vec<i32> {
            a
        }

        async fn test_arg(&self, input: Vec<i32>) -> Vec<i32> {
            input
        }

        async fn test_input<'a>(&self, input: MyInput) -> Vec<i32> {
            input.value
        }
    }

    let schema = Schema::new(
        Root {
            value_vec: vec![1, 2, 3, 4, 5],
            value_hash_set: vec![1, 2, 3, 4, 5].into_iter().collect(),
            value_btree_set: vec![1, 2, 3, 4, 5].into_iter().collect(),
            value_linked_list: vec![1, 2, 3, 4, 5].into_iter().collect(),
            value_vec_deque: vec![1, 2, 3, 4, 5].into_iter().collect(),
        },
        EmptyMutation,
        EmptySubscription,
    );
    let json_value: serde_json::Value = vec![1, 2, 3, 4, 5].into();
    let query = format!(
        r#"{{
            valueVec
            valueSlice
            valueLinkedList
            valueHashSet
            valueBtreeSet
            valueVecDeque
            testArg(input: {0})
            testInput(input: {{value: {0}}})
            valueInputSlice1: valueInputSlice(a: [1, 2, 3])
            valueInputSlice2: valueInputSlice(a: 55)
            }}
            "#,
        json_value
    );
    let mut res = schema.execute(&query).await.data;

    if let Value::Object(obj) = &mut res {
        if let Some(Value::List(array)) = obj.get_mut("valueHashSet") {
            array.sort_by(|a, b| {
                if let (Value::Number(a), Value::Number(b)) = (a, b) {
                    if let (Some(a), Some(b)) = (a.as_i64(), b.as_i64()) {
                        return a.cmp(&b);
                    }
                }
                Ordering::Less
            });
        }
    }

    assert_eq!(
        res,
        value!({
            "valueVec": vec![1, 2, 3, 4, 5],
            "valueSlice": vec![1, 2, 3, 4, 5],
            "valueLinkedList": vec![1, 2, 3, 4, 5],
            "valueHashSet": vec![1, 2, 3, 4, 5],
            "valueBtreeSet": vec![1, 2, 3, 4, 5],
            "valueVecDeque": vec![1, 2, 3, 4, 5],
            "testArg": vec![1, 2, 3, 4, 5],
            "testInput": vec![1, 2, 3, 4, 5],
            "valueInputSlice1": vec![1, 2, 3],
            "valueInputSlice2": vec![55],
        })
    );
}

#[tokio::test]
pub async fn test_array_type() {
    struct Query;

    #[Object]
    impl Query {
        async fn values(&self) -> [i32; 6] {
            [1, 2, 3, 4, 5, 6]
        }

        async fn array_input(&self, values: [i32; 6]) -> [i32; 6] {
            assert_eq!(values, [1, 2, 3, 4, 5, 6]);
            values
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert_eq!(
        schema
            .execute("{ values }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "values": [1, 2, 3, 4, 5, 6]
        })
    );

    assert_eq!(
        schema
            .execute("{ arrayInput(values: [1, 2, 3, 4, 5, 6]) }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "arrayInput": [1, 2, 3, 4, 5, 6]
        })
    );

    assert_eq!(
        schema
            .execute("{ arrayInput(values: [1, 2, 3, 4, 5]) }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: r#"Failed to parse "[Int!]": Expected input type "[Int; 6]", found [Int; 5]."#
                .to_owned(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 22,
            }],
            path: vec![PathSegment::Field("arrayInput".to_owned())],
            extensions: None,
        }],
    );
}
