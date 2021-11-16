use crate::{InputType, InputValueError};

pub fn min_length<T: AsRef<str> + InputType>(
    value: &T,
    len: usize,
) -> Result<(), InputValueError<T>> {
    if value.as_ref().len() >= len {
        Ok(())
    } else {
        Err(format!(
            "the string length is {}, must be greater than or equal to {}",
            value.as_ref().len(),
            len
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_length() {
        assert!(min_length(&"ab".to_string(), 3).is_err());
        assert!(min_length(&"abc".to_string(), 3).is_ok());
        assert!(min_length(&"abcd".to_string(), 3).is_ok());
    }
}
