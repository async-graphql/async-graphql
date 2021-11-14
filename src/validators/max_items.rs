use std::ops::Deref;

use crate::{InputType, InputValueError};

pub async fn max_items<T: Deref<Target = [E]> + InputType, E>(
    value: &T,
    len: usize,
) -> Result<(), InputValueError<T>> {
    if value.deref().len() <= len {
        Ok(())
    } else {
        Err(format!(
            "the value length is {}, must be less than or equal to {}",
            value.deref().len(),
            len
        )
        .into())
    }
}
