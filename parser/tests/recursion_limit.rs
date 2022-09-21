use async_graphql_parser::*;

#[test]
fn test_recursion_limit() {
    let depth = 65;
    let field = "a {".repeat(depth) + &"}".repeat(depth);
    let query = format!("query {{ {} }}", field.replace("{}", "{b}"));
    assert_eq!(
        parse_query(query).unwrap_err(),
        Error::RecursionLimitExceeded
    );
}

#[test]
fn test_issue_1039() {
    let query = r#"
      fragment onboardingFull on OnboardingState {
        license
      }
      
      query globalConfig {
        globalConfig {
          onboarding {
            ...onboardingFull
          }
        }
      }
    "#;
    parse_query(query).unwrap();
}
