use bytes::{Buf, Bytes};
use futures::task::{Context, Poll};
use futures::{AsyncRead, Stream, StreamExt};
use std::io::{Error, ErrorKind, Result};
use std::pin::Pin;

/// An Adapter for bytes stream to `AsyncRead`
pub struct StreamBody<S> {
    s: S,
    remaining_bytes: Option<Bytes>,
}

impl<S> StreamBody<S> {
    #[allow(missing_docs)]
    pub fn new(s: S) -> Self {
        Self {
            s,
            remaining_bytes: None,
        }
    }
}

impl<S, E, D> AsyncRead for StreamBody<S>
where
    D: Buf,
    S: Stream<Item = std::result::Result<D, E>> + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        loop {
            if let Some(bytes) = &mut self.remaining_bytes {
                let data = bytes.split_to(buf.len().min(bytes.len()));
                buf[..data.len()].copy_from_slice(&data);
                if !bytes.has_remaining() {
                    self.remaining_bytes = None;
                }
                return Poll::Ready(Ok(data.len()));
            } else {
                match self.s.poll_next_unpin(cx) {
                    Poll::Ready(Some(Ok(mut bytes))) => {
                        self.remaining_bytes = Some(bytes.to_bytes());
                    }
                    Poll::Ready(Some(Err(_))) => {
                        return Poll::Ready(Err(Error::from(ErrorKind::InvalidData)))
                    }
                    Poll::Ready(None) => return Poll::Ready(Ok(0)),
                    Poll::Pending => return Poll::Pending,
                }
            }
        }
    }
}
