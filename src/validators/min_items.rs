use std::ops::Deref;

use crate::{InputType, InputValueError};

pub fn min_items<T: Deref<Target = [E]> + InputType, E>(
    value: &T,
    len: usize,
) -> Result<(), InputValueError<T>> {
    if value.deref().len() >= len {
        Ok(())
    } else {
        Err(format!(
            "the value length is {}, must be greater than or equal to {}",
            value.deref().len(),
            len
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_items() {
        assert!(min_items(&vec![1, 2], 3).is_err());
        assert!(min_items(&vec![1, 2, 3], 3).is_ok());
        assert!(min_items(&vec![1, 2, 3, 4], 3).is_ok());
    }
}
