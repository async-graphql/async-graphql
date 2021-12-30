use crate::{InputType, InputValueError};
use zxcvbn::{zxcvbn, ZxcvbnError};

pub fn min_password_strength<T: AsRef<str> + InputType>(
    value: &T,
    min_score: u8,
) -> Result<(), InputValueError<T>> {
    match zxcvbn(value.as_ref(), &[]) {
        Ok(password_strength) => {
            if password_strength.score() < min_score {
                Err("password is too weak".into())
            } else {
                Ok(())
            }
        }
        Err(ZxcvbnError::BlankPassword) => Err("password is too weak".into()),
        _ => Err("error processing password strength".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_password_strength() {
        assert!(min_password_strength(&"password".to_string(), 3).is_err());
        assert!(min_password_strength(&"query".to_string(), 3).is_err());
        assert!(min_password_strength(&"P@ssword1".to_string(), 3).is_err());
        assert!(min_password_strength(&"".to_string(), 3).is_err());

        assert!(min_password_strength(&"Some!Secure!Password".to_string(), 3).is_ok());
    }
}
