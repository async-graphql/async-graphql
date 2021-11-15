use num_traits::AsPrimitive;

use crate::{InputType, InputValueError};

pub fn multiple_of<T: AsPrimitive<f64> + InputType>(
    value: &T,
    n: f64,
) -> Result<(), InputValueError<T>> {
    if value.as_() % n as f64 == 0.0 {
        Ok(())
    } else {
        Err(format!("the value must be a multiple of {}.", n).into())
    }
}
