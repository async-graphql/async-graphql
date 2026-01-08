use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

/// A runtime-agnostic delay.
///
/// This helps abstract over different runtime implementations and only uses
/// [`async_io::Timer`] as a fallback implementation. [`async_io::Timer`] is generally
/// avoided, since most runtimes provide their own timer implementations without needing to spawn
/// a new helper thread.
pub struct Delay(DelayInner);

enum DelayInner {
    #[cfg(feature = "tokio-time")]
    Tokio(Pin<Box<tokio::time::Sleep>>),
    #[cfg(feature = "smol")]
    Smol(Pin<Box<smol::Timer>>),
    AsyncIo(Pin<Box<async_io::Timer>>),
}

impl Delay {
    /// Creates a new `Delay` instance that waits for the specified duration.
    pub fn new(duration: Duration) -> Self {
        #[cfg(feature = "tokio-time")]
        if tokio::runtime::Handle::try_current().is_ok() {
            return Self(DelayInner::Tokio(Box::pin(tokio::time::sleep(duration))));
        }

        #[cfg(feature = "smol")]
        {
            return Self(DelayInner::Smol(Box::pin(smol::Timer::after(duration))));
        }

        // Fallback
        #[allow(unreachable_code)]
        Self(DelayInner::AsyncIo(Box::pin(async_io::Timer::after(
            duration,
        ))))
    }

    /// Resets the delay to wait for the specified duration.
    pub fn reset(&mut self, duration: Duration) {
        match &mut self.0 {
            #[cfg(feature = "tokio-time")]
            DelayInner::Tokio(sleep) => {
                sleep.as_mut().reset(tokio::time::Instant::now() + duration);
            }
            #[cfg(feature = "smol")]
            DelayInner::Smol(timer) => {
                timer.as_mut().set_after(duration);
            }
            DelayInner::AsyncIo(timer) => {
                timer.as_mut().set_after(duration);
            }
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &mut self.0 {
            #[cfg(feature = "tokio-time")]
            DelayInner::Tokio(sleep) => sleep.as_mut().poll(cx),
            #[cfg(feature = "smol")]
            DelayInner::Smol(timer) => timer.as_mut().poll(cx).map(|_| ()),
            DelayInner::AsyncIo(timer) => timer.as_mut().poll(cx).map(|_| ()),
        }
    }
}

// Ensure Delay is Unpin
impl Unpin for Delay {}
