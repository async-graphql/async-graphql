use std::fmt::Display;
use std::ops::Rem;

use num_traits::{AsPrimitive, Zero};

use crate::{InputType, InputValueError};

pub fn multiple_of<T, N>(value: &T, n: N) -> Result<(), InputValueError<T>>
where
    T: AsPrimitive<N> + InputType,
    N: Rem<Output = N> + Zero + Display + Copy + PartialEq + 'static,
{
    if value.as_() % n == N::zero() {
        Ok(())
    } else {
        Err(format!("the value must be a multiple of {}.", n).into())
    }
}
