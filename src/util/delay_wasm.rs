use std::{
    future::Future,
    pin::Pin,
    task::{self, Poll},
    time::Duration,
};

pub struct Delay {
    _priv: (),
}

impl Delay {
    pub fn new(_: Duration) -> Self {
        unimplemented!();
    }

    pub fn reset(&mut self, _: Duration) {
        unimplemented!();
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut task::Context<'_>) -> Poll<Self::Output> {
        unimplemented!();
    }
}
