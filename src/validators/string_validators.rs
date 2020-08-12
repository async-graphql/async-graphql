use crate::validators::InputValueValidator;
use crate::Value;
use once_cell::sync::Lazy;
use regex::Regex;

/// String minimum length validator
pub struct StringMinLength {
    /// Must be greater than or equal to this value.
    pub length: i32,
}

impl InputValueValidator for StringMinLength {
    fn is_valid(&self, value: &Value) -> Result<(), String> {
        if let Value::String(s) = value {
            if s.len() < self.length as usize {
                Err(format!(
                    "the value length is {}, must be greater than or equal to {}",
                    s.len(),
                    self.length
                ))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

/// String maximum length validator
pub struct StringMaxLength {
    /// Must be less than or equal to this value.
    pub length: i32,
}

impl InputValueValidator for StringMaxLength {
    fn is_valid(&self, value: &Value) -> Result<(), String> {
        if let Value::String(s) = value {
            if s.len() > self.length as usize {
                Err(format!(
                    "the value length is {}, must be less than or equal to {}",
                    s.len(),
                    self.length
                ))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

static EMAIL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new("^(([0-9A-Za-z!#$%&'*+-/=?^_`{|}~&&[^@]]+)|(\"([0-9A-Za-z!#$%&'*+-/=?^_`{|}~ \"(),:;<>@\\[\\\\\\]]+)\"))@").unwrap()
});

/// Email validator
pub struct Email {}

impl InputValueValidator for Email {
    fn is_valid(&self, value: &Value) -> Result<(), String> {
        if let Value::String(s) = value {
            if !EMAIL_RE.is_match(s) {
                Err("invalid email format".to_string())
            } else {
                Ok(())
            }
        } else {
            Ok(())
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
    fn is_valid(&self, value: &Value) -> Result<(), String> {
        if let Value::String(s) = value {
            if self.colon {
                if !MAC_ADDRESS_RE.is_match(s) {
                    Err("invalid MAC format".to_string())
                } else {
                    Ok(())
                }
            } else if !MAC_ADDRESS_NO_COLON_RE.is_match(s) {
                Err("invalid MAC format".to_string())
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}
