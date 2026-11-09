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

fn nested_value(depth: usize, open: &str, close: &str) -> String {
    format!("{}1{}", open.repeat(depth), close.repeat(depth))
}

#[test]
fn test_value_recursion_limit() {
    for (open, close) in [("{b: ", "}"), ("[", "]")] {
        let query = |depth| format!("{{ a(x: {}) }}", nested_value(depth, open, close));
        parse_query(query(64)).unwrap();
        assert_eq!(
            parse_query(query(65)).unwrap_err(),
            Error::RecursionLimitExceeded
        );
    }
}

#[test]
fn test_const_value_recursion_limit() {
    let query = |depth| format!("query($v: T = {}) {{ a }}", nested_value(depth, "[", "]"));
    parse_query(query(64)).unwrap();
    assert_eq!(
        parse_query(query(65)).unwrap_err(),
        Error::RecursionLimitExceeded
    );

    let schema = |depth| {
        format!(
            "type Query {{ a(x: T = {}): Int }}",
            nested_value(depth, "{b: ", "}")
        )
    };
    parse_schema(schema(64)).unwrap();
    assert_eq!(
        parse_schema(schema(65)).unwrap_err(),
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
