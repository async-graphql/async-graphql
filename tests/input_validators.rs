use async_graphql::validators::StringMinLength;
use async_graphql::*;

#[async_std::test]
pub async fn test_string_min_length() {
    struct Query1;

    #[Object]
    impl Query1 {
        async fn test(&self, #[arg(validator(StringMinLength(length = "6")))] _id: String) -> bool {
            true
        }
    }
}
