use crate::validators::InputValueValidator;
use graphql_parser::query::Value;

/// Integer range validator
pub struct IntRange {
    /// Minimum value, including this value.
    pub min: i64,

    /// Maximum value, including this value.
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
            None
        }
    }
}

/// Integer less then validator
pub struct IntLessThan {
    /// Less then this value.
    pub value: i64,
}

impl InputValueValidator for IntLessThan {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::Int(n) = value {
            if n.as_i64().unwrap() >= self.value {
                Some(format!(
                    "the value is {}, must be less than {}",
                    n.as_i64().unwrap(),
                    self.value
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Integer greater then validator
pub struct IntGreaterThan {
    /// Greater then this value.
    pub value: i64,
}

impl InputValueValidator for IntGreaterThan {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::Int(n) = value {
            if n.as_i64().unwrap() <= self.value {
                Some(format!(
                    "the value is {}, must be greater than {}",
                    n.as_i64().unwrap(),
                    self.value
                ))
            } else {
                None
            }
        } else {
            Some("expected type \"Int\"".to_string())
        }
    }
}

/// Integer nonzero validator
pub struct IntNonZero {}

impl InputValueValidator for IntNonZero {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::Int(n) = value {
            if n.as_i64().unwrap() == 0 {
                Some(format!(
                    "the value is {}, but must be nonzero",
                    n.as_i64().unwrap(),
                ))
            } else {
                None
            }
        } else {
            Some("expected type \"Int\"".to_string())
        }
    }
}
