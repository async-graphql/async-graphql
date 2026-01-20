use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use gloo_timers::future::TimeoutFuture;
use pin_project_lite::pin_project;
use send_wrapper::SendWrapper;

pin_project! {
    /// A Wasm-only delay.
    ///
    /// This is a fallback for the regular `Delay` from the `delay` module.
    /// It uses gloo_timers as the underlying timer.
    pub(crate) struct Delay {
        #[pin]
        inner: SendWrapper<TimeoutFuture>
    }
}

impl Delay {
    /// Creates a new `Delay` instance that waits for the specified duration.
    pub fn new(duration: Duration) -> Self {
        Self {
            inner: SendWrapper::new(TimeoutFuture::new(duration.as_millis() as u32)),
        }
    }

    /// Resets the delay to wait for the specified duration.
    pub fn reset(self: Pin<&mut Self>, duration: Duration) {
        let mut this = self.project();
        this.inner.set(SendWrapper::new(TimeoutFuture::new(
            duration.as_millis() as u32
        )));
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.inner.poll(cx)
    }
}
