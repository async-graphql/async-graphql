use async_graphql::*;

macro_rules! test_scalars {
    ($test_name:ident, $ty:ty, $value:expr, $res_value:expr) => {
        #[async_std::test]
        pub async fn $test_name() {
            #[InputObject]
            struct MyInput {
                value: $ty,
            }

            struct Root {
                value: $ty,
            }

            #[Object]
            impl Root {
                async fn value(&self) -> $ty {
                    self.value
                }

                async fn test_arg(&self, input: $ty) -> $ty {
                    input
                }

                async fn test_input(&self, input: MyInput) -> $ty {
                    input.value
                }
            }

            let schema = Schema::new(Root { value: $value }, EmptyMutation, EmptySubscription);
            let json_value: serde_json::Value = $value.into();
            let query = format!("{{ value testArg(input: {0}) testInput(input: {{value: {0}}}) }}", json_value);
            assert_eq!(
                schema.execute(&query).await.unwrap().data,
                serde_json::json!({ "value": $res_value, "testArg": $res_value, "testInput": $res_value })
            );
        }
    };
}

test_scalars!(test_i8_scalar, i8, 10, 10);
test_scalars!(test_i16_scalar, i16, 10, 10);
test_scalars!(test_i32_scalar, i32, 10, 10);
test_scalars!(test_u8_scalar, u8, 10, 10);
test_scalars!(test_u16_scalar, u16, 10, 10);
test_scalars!(test_u32_scalar, u32, 10, 10);
test_scalars!(test_bool_scalar, bool, true, true);
test_scalars!(test_f32_scalar, f32, 10.5, 10.5);
test_scalars!(test_f64_scalar, f32, 10.5, 10.5);

test_scalars!(test_i64_scalar, i64, 10, "10");
test_scalars!(test_u64_scalar, u64, 10, "10");
