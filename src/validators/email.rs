use fast_chemail::is_valid_email;

use crate::{InputType, InputValueError};

pub fn email<T: AsRef<str> + InputType>(value: &T) -> Result<(), InputValueError<T>> {
    if is_valid_email(value.as_ref()) {
        Ok(())
    } else {
        Err("invalid email".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email() {
        assert!(email(&"joe@example.com".to_string()).is_ok());
        assert!(email(&"joe.test@example.com".to_string()).is_ok());
        assert!(email(&"email@example-one.com".to_string()).is_ok());
        assert!(email(&"1234567890@example.com".to_string()).is_ok());

        assert!(email(&"plainaddress".to_string()).is_err());
        assert!(email(&"@example.com".to_string()).is_err());
        assert!(email(&"email.example.com".to_string()).is_err());
    }
}
