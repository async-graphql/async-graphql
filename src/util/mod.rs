#[cfg(not(target_arch = "wasm32"))]
mod delay;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use delay::Delay;

#[cfg(target_arch = "wasm32")]
mod delay_wasm;
#[cfg(target_arch = "wasm32")]
pub(crate) use delay_wasm::Delay;
