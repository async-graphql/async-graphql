use num_traits::AsPrimitive;

use crate::{InputType, InputValueError};

pub async fn minimum<T: AsPrimitive<f64> + InputType>(
    value: &T,
    n: f64,
) -> Result<(), InputValueError<T>> {
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
