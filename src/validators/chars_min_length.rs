use crate::{InputType, InputValueError};

pub fn chars_min_length<T: AsRef<str> + InputType>(
    value: &T,
    len: usize,
) -> Result<(), InputValueError<T>> {
    if value.as_ref().chars().count() >= len {
        Ok(())
    } else {
        Err(format!(
            "the chars length is {}, must be greater than or equal to {}",
            value.as_ref().chars().count(),
            len
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chars_min_length() {
        assert!(chars_min_length(&"你好".to_string(), 3).is_err());
        assert!(chars_min_length(&"你好啊".to_string(), 3).is_ok());
        assert!(chars_min_length(&"嗨你好啊".to_string(), 3).is_ok());
    }
}
