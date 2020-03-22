use crate::validators::InputValueValidator;
use graphql_parser::schema::Value;
use once_cell::sync::Lazy;
use regex::Regex;

/// String minimum length validator
pub struct StringMinLength {
    /// Must be greater than or equal to this value.
    pub length: usize,
}

impl InputValueValidator for StringMinLength {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::String(s) = value {
            if s.len() < self.length {
                Some(format!(
                    "The value length is {}, but the length must be greater than or equal to {}",
                    s.len(),
                    self.length
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// String maximum length validator
pub struct StringMaxLength {
    /// Must be less than or equal to this value.
    pub length: usize,
}

impl InputValueValidator for StringMaxLength {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::String(s) = value {
            if s.len() > self.length {
                Some(format!(
                    "The value length is {}, but the length must be less than or equal to {}",
                    s.len(),
                    self.length
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
}

static EMAIL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new("^(([0-9A-Za-z!#$%&'*+-/=?^_`{|}~&&[^@]]+)|(\"([0-9A-Za-z!#$%&'*+-/=?^_`{|}~ \"(),:;<>@\\[\\\\\\]]+)\"))@").unwrap()
});

/// Email validator
pub struct Email {}

impl InputValueValidator for Email {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::String(s) = value {
            if !EMAIL_RE.is_match(s) {
                Some("invalid email format".to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
}

static MAC_ADDRESS_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new("^([0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}$").unwrap());
static MAC_ADDRESS_NO_COLON_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new("^[0-9a-fA-F]{12}$").unwrap());

/// MAC address validator
pub struct MAC {
    /// Must include colon.
    pub colon: bool,
}

impl InputValueValidator for MAC {
    fn is_valid(&self, value: &Value) -> Option<String> {
        if let Value::String(s) = value {
            if self.colon {
                if !MAC_ADDRESS_RE.is_match(s) {
                    Some("invalid email format".to_string())
                } else {
                    None
                }
            } else if !MAC_ADDRESS_NO_COLON_RE.is_match(s) {
                Some("invalid email format".to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
}
