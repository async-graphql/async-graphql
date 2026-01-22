//! Runtime abstraction traits

use std::time::Duration;

#[cfg(feature = "tokio")]
mod tokio {
    use std::time::Duration;

    use futures_util::{
        FutureExt,
        future::BoxFuture,
        task::{FutureObj, Spawn, SpawnError},
    };
    use tokio::runtime::Handle;

    use crate::runtime::Timer;

    /// A Tokio-backed implementation of [`Spawn`]
    ///
    /// We use this abstraction across the crate for spawning tasks onto the
    /// runtime
    pub struct TokioSpawner {
        handle: Handle,
    }

    impl TokioSpawner {
        /// Construct a spawner that obtains a handle of the current runtime
        ///
        /// # Panics
        ///
        /// Panics when used outside of the context of a Tokio runtime
        pub fn current() -> Self {
            Self::with_handle(Handle::current())
        }

        /// Construct a spawner with a handle of a specific runtime
        pub fn with_handle(handle: Handle) -> Self {
            Self { handle }
        }
    }

    impl Spawn for TokioSpawner {
        fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
            self.handle.spawn(future);
            Ok(())
        }
    }

    /// A Tokio-backed implementation of the [`Timer`] trait
    #[derive(Default)]
    pub struct TokioTimer {
        _priv: (),
    }

    impl Timer for TokioTimer {
        fn delay(&self, duration: Duration) -> BoxFuture<'static, ()> {
            tokio::time::sleep(duration).boxed()
        }
    }
}
use futures_util::future::BoxFuture;

#[cfg(feature = "tokio")]
pub use self::tokio::{TokioSpawner, TokioTimer};

/// Timing facilities required by parts of the crate
///
/// The purpose is to make async-graphql integrate nicely with whatever
/// environment you're in.
///
/// Be it Tokio, smol, or even the browser.
pub trait Timer: Send + Sync + 'static {
    /// Returns a future that resolves after the specified duration
    fn delay(&self, duration: Duration) -> BoxFuture<'static, ()>;
}

const _: Option<&dyn Timer> = None;
