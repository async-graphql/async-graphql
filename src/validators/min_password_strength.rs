use std::convert::TryFrom;

use zxcvbn::zxcvbn;

use crate::{InputType, InputValueError};

pub fn min_password_strength<T: AsRef<str> + InputType>(
    value: &T,
    min_score: u8,
) -> Result<(), InputValueError<T>> {
    let password_strength = zxcvbn(value.as_ref(), &[]);
    if password_strength.score() < zxcvbn::Score::try_from(min_score).expect("invalid score") {
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
