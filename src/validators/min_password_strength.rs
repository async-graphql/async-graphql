use zxcvbn::{zxcvbn, Score};

use crate::{InputType, InputValueError};

pub fn min_password_strength<T: AsRef<str> + InputType>(
    value: &T,
    min_score: u8,
) -> Result<(), InputValueError<T>> {
    let min_score = match min_score {
        0 => Score::Zero,
        1 => Score::One,
        2 => Score::Two,
        3 => Score::Three,
        _ => Score::Four,
    };

    let score = zxcvbn(value.as_ref(), &[]).score();

    if score < min_score {
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
