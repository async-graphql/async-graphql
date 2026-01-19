use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use pin_project_lite::pin_project;

pin_project! {
    /// A runtime-agnostic delay.
    ///
    /// This helps abstract over different runtime implementations and only uses
    /// [`async_io::Timer`] as a fallback implementation. [`async_io::Timer`] is
    /// generally avoided, since most runtimes provide their own timer
    /// implementations without needing to spawn a new helper thread.
    pub(crate) struct Delay {
        #[pin]
        inner: DelayInner
    }
}

pin_project_cfg! {
    #[project = DelayInnerProj]
    pub(crate) enum DelayInner {
        #[cfg(feature = "tokio-timer")]
        Tokio { #[pin] fut: tokio::time::Sleep, },
        AsyncIo { #[pin] fut: async_io::Timer, },
    }
}

impl Delay {
    /// Creates a new `Delay` instance that waits for the specified duration.
    pub fn new(duration: Duration) -> Self {
        #[cfg(feature = "tokio-timer")]
        if tokio::runtime::Handle::try_current().is_ok() {
            return Self { inner: DelayInner::Tokio {
                fut: tokio::time::sleep(duration),
            }};
        }

        // Fallback
        #[allow(unreachable_code)]
        Self { inner: DelayInner::AsyncIo {
            fut: async_io::Timer::after(duration),
        }}
    }

    /// Resets the delay to wait for the specified duration.
    pub fn reset(self: Pin<&mut Self>, duration: Duration) {
        let this = self.project();
        match this.inner.project() {
            #[cfg(feature = "tokio-timer")]
            DelayInnerProj::Tokio { fut: sleep } => {
                sleep.reset(tokio::time::Instant::now() + duration);
            }
            DelayInnerProj::AsyncIo { fut: mut timer } => {
                timer.set_after(duration);
            }
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.project() {
            #[cfg(feature = "tokio-timer")]
            DelayInnerProj::Tokio { fut: sleep } => sleep.poll(cx),
            DelayInnerProj::AsyncIo { fut: timer } => timer.poll(cx).map(|_| ()),
        }
    }
}
