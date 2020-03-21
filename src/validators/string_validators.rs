use crate::validators::InputValueValidator;
use graphql_parser::schema::Value;
use once_cell::sync::Lazy;
use regex::Regex;

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
            Some("expected type \"String\"".to_string())
        }
    }
}

static MAC_ADDRESS_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new("^([0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}$").unwrap());
static MAC_ADDRESS_NO_COLON_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new("^[0-9a-fA-F]{12}$").unwrap());

/// MAC address validator
pub struct MAC {
    /// Must include colon
    colon: bool,
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
            Some("expected type \"String\"".to_string())
        }
    }
}
