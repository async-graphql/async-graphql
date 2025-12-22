use zxcvbn::zxcvbn;

use crate::{InputType, InputValueError};

pub fn min_password_strength<T: AsRef<str> + InputType>(
    value: &T,
    min_score: u8,
) -> Result<(), InputValueError<T>> {
    let entropy = zxcvbn(value.as_ref(), &[]);
    if u8::from(entropy.score()) < min_score {
        Err("password is too weak".into())
    } else {
        Ok(())
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
