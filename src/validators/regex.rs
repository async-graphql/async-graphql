use regex::Regex;

use crate::{InputType, InputValueError};

pub fn regex<T: AsRef<str> + InputType>(
    value: &T,
    regex: &'static str,
) -> Result<(), InputValueError<T>> {
    if let Ok(true) = Regex::new(regex).map(|re| re.is_match(value.as_ref())) {
        Ok(())
    } else {
        Err("value is valid".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url() {
        assert!(regex(&"123".to_string(), "^[0-9]+$").is_ok());
        assert!(regex(&"12a3".to_string(), "^[0-9]+$").is_err());
    }
}
