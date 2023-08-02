use std::{
    task::{Context, Poll},
    time::Duration,
};

pub trait Timer {
    fn new(interval: Duration) -> Self;

    fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<()>;

    fn reset(&mut self);
}

pub struct DefaultTimer;

impl Timer for DefaultTimer {
    fn new(_interval: Duration) -> Self {
        Self
    }

    fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        Poll::Pending
    }

    fn reset(&mut self) {}
}
