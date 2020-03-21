use crate::validators::InputValueValidator;
use graphql_parser::query::Value;

/// Integer range validator
pub struct IntRange {
    /// Minimum value, including this value
    pub min: i64,

    /// Maximum value, including this value
    pub max: i64,
}

impl InputValueValidator for IntRange {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::Int(n) = value {
            if n.as_i64().unwrap() < self.min || n.as_i64().unwrap() > self.max {
                Some(format!(
                    "the value is {}, but the range must be between {} and {}",
                    n.as_i64().unwrap(),
                    self.min,
                    self.max
                ))
            } else {
                None
            }
        } else {
            Some("expected type \"Int\"".to_string())
        }
    }
}
