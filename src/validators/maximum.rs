use std::fmt::Display;

use num_traits::AsPrimitive;

use crate::{InputType, InputValueError};

pub fn maximum<T, N>(value: &T, n: N) -> Result<(), InputValueError<T>>
where
    T: AsPrimitive<N> + InputType,
    N: PartialOrd + Display + Copy + 'static,
{
    if value.as_() <= n {
        Ok(())
    } else {
        Err(format!(
            "the value is {}, must be less than or equal to {}",
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
    fn test_maximum() {
        assert!(maximum(&99, 100).is_ok());
        assert!(maximum(&100, 100).is_ok());
        assert!(maximum(&101, 100).is_err());
    }
}
