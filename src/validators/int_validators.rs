use crate::validators::InputValueValidator;
use crate::Value;

/// Integer range validator
pub struct IntRange {
    /// Minimum value, including this value.
    pub min: i32,

    /// Maximum value, including this value.
    pub max: i32,
}

impl InputValueValidator for IntRange {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::Int(n) = value {
            if *n < self.min || *n > self.max {
                Some(format!(
                    "the value is {}, must be between {} and {}",
                    *n, self.min, self.max
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
    pub value: i32,
}

impl InputValueValidator for IntLessThan {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::Int(n) = value {
            if *n >= self.value {
                Some(format!(
                    "the value is {}, must be less than {}",
                    *n, self.value
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
    pub value: i32,
}

impl InputValueValidator for IntGreaterThan {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::Int(n) = value {
            if *n <= self.value {
                Some(format!(
                    "the value is {}, must be greater than {}",
                    *n, self.value
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Integer nonzero validator
pub struct IntNonZero {}

impl InputValueValidator for IntNonZero {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::Int(n) = value {
            if *n == 0 {
                Some(format!("the value is {}, must be nonzero", *n,))
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Integer equal validator
pub struct IntEqual {
    /// equal this value.
    pub value: i32,
}

impl InputValueValidator for IntEqual {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::Int(n) = value {
            if *n != self.value {
                Some(format!(
                    "the value is {}, must be equal to {}",
                    *n, self.value
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
}
