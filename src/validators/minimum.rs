use std::fmt::Display;

use num_traits::AsPrimitive;

use crate::{InputType, InputValueError};

pub fn minimum<T, N>(value: &T, n: N) -> Result<(), InputValueError<T>>
where
    T: AsPrimitive<N> + InputType,
    N: PartialOrd + Display + Copy + 'static,
{
    if value.as_() >= n {
        Ok(())
    } else {
        Err(format!(
            "the value is {}, must be greater than or equal to {}",
            value.as_(),
            n
        )
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimum() {
        assert!(minimum(&99, 100).is_err());
        assert!(minimum(&100, 100).is_ok());
        assert!(minimum(&101, 100).is_ok());
    }
}
