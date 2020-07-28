use crate::validators::InputValueValidator;
use crate::Value;

/// Integer range validator
pub struct IntRange {
    /// Minimum value, including this value.
    pub min: i64,

    /// Maximum value, including this value.
    pub max: i64,
}

impl InputValueValidator for IntRange {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::Number(n) = value {
            if let Some(n) = n.as_i64() {
                if n < self.min || n > self.max {
                    return Some(format!(
                        "the value is {}, must be between {} and {}",
                        n, self.min, self.max
                    ));
                }
            }
        }
        None
    }
}

/// Integer less then validator
pub struct IntLessThan {
    /// Less then this value.
    pub value: i64,
}

impl InputValueValidator for IntLessThan {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::Number(n) = value {
            if let Some(n) = n.as_i64() {
                if n >= self.value {
                    return Some(format!(
                        "the value is {}, must be less than {}",
                        n, self.value
                    ));
                }
            }
        }
        None
    }
}

/// Integer greater then validator
pub struct IntGreaterThan {
    /// Greater then this value.
    pub value: i64,
}

impl InputValueValidator for IntGreaterThan {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::Number(n) = value {
            if let Some(n) = n.as_i64() {
                if n <= self.value {
                    return Some(format!(
                        "the value is {}, must be greater than {}",
                        n, self.value
                    ));
                }
            }
        }
        None
    }
}

/// Integer nonzero validator
pub struct IntNonZero {}

impl InputValueValidator for IntNonZero {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::Number(n) = value {
            if let Some(n) = n.as_i64() {
                if n == 0 {
                    return Some(format!("the value is {}, must be nonzero", n));
                }
            }
        }
        None
    }
}

/// Integer equal validator
pub struct IntEqual {
    /// equal this value.
    pub value: i64,
}

impl InputValueValidator for IntEqual {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::Number(n) = value {
            if let Some(n) = n.as_i64() {
                if n != self.value {
                    return Some(format!(
                        "the value is {}, must be equal to {}",
                        n, self.value
                    ));
                }
            }
        }
        None
    }
}
