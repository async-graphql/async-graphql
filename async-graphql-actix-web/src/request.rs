use actix_web::web::Payload;
use async_graphql::http::GQLHttpRequest;
use bytes::{Buf, Bytes};
use futures::io::{Error, ErrorKind, Result};
use futures::task::{Context, Poll};
use futures::{AsyncRead, StreamExt};
use std::pin::Pin;

pub struct RequestWrapper(pub Option<String>, pub Payload);

unsafe impl Send for RequestWrapper {}
unsafe impl Sync for RequestWrapper {}

impl GQLHttpRequest for RequestWrapper {
    type Body = PayloadReader;

    fn content_type(&self) -> Option<&str> {
        self.0.as_deref()
    }

    fn into_body(self) -> Self::Body {
        PayloadReader {
            payload: self.1,
            remain_bytes: None,
        }
    }
}

pub struct PayloadReader {
    payload: Payload,
    remain_bytes: Option<Bytes>,
}

unsafe impl Send for PayloadReader {}
unsafe impl Sync for PayloadReader {}

impl AsyncRead for PayloadReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        loop {
            if let Some(bytes) = &mut self.remain_bytes {
                let data = bytes.split_to(buf.len().min(bytes.len()));
                buf[..data.len()].copy_from_slice(&data);
                if !bytes.has_remaining() {
                    self.remain_bytes = None;
                }
                return Poll::Ready(Ok(data.len()));
            } else {
                match self.payload.poll_next_unpin(cx) {
                    Poll::Ready(Some(Ok(bytes))) => {
                        self.remain_bytes = Some(bytes);
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
