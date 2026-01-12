use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

/// A runtime-agnostic delay.
///
/// This helps abstract over different runtime implementations and only uses
/// [`async_io::Timer`] as a fallback implementation. [`async_io::Timer`] is
/// generally avoided, since most runtimes provide their own timer
/// implementations without needing to spawn a new helper thread.
pub(crate) struct Delay(DelayInner);

enum DelayInner {
    #[cfg(feature = "tokio-timer")]
    Tokio(Pin<Box<tokio::time::Sleep>>),
    AsyncIo(Pin<Box<async_io::Timer>>),
}

impl Delay {
    /// Creates a new `Delay` instance that waits for the specified duration.
    pub fn new(duration: Duration) -> Self {
        #[cfg(feature = "tokio-timer")]
        if tokio::runtime::Handle::try_current().is_ok() {
            return Self(DelayInner::Tokio(Box::pin(tokio::time::sleep(duration))));
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
            #[cfg(feature = "tokio-timer")]
            DelayInner::Tokio(sleep) => {
                sleep.as_mut().reset(tokio::time::Instant::now() + duration);
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
            #[cfg(feature = "tokio-timer")]
            DelayInner::Tokio(sleep) => sleep.as_mut().poll(cx),
            DelayInner::AsyncIo(timer) => timer.as_mut().poll(cx).map(|_| ()),
        }
    }
}

// Ensure Delay is Unpin
impl Unpin for Delay {}
