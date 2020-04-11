use futures::io::ErrorKind;
use futures::task::{Context, Poll};
use futures::{AsyncBufRead, Future};
use std::io::{Error, Result};
use std::pin::Pin;

pub trait AsyncTokenReader: AsyncBufRead {
    fn read_until_token<'a>(
        &'a mut self,
        token: &'a [u8],
        buf: &'a mut [u8],
        state: &'a mut ReadUntilState,
    ) -> ReadUntilToken<'a, Self> {
        ReadUntilToken {
            reader: self,
            token,
            buf,
            state,
        }
    }

    fn except_token<'a>(&'a mut self, token: &'a [u8]) -> ExceptToken<'a, Self> {
        ExceptToken {
            reader: self,
            token,
            match_size: 0,
        }
    }
}

impl<R: AsyncBufRead> AsyncTokenReader for R {}

#[derive(Default)]
pub struct ReadUntilState {
    match_size: usize,
    consume_token: Option<(usize, usize)>,
}

pub struct ReadUntilToken<'a, R: ?Sized> {
    reader: &'a mut R,
    token: &'a [u8],
    buf: &'a mut [u8],
    state: &'a mut ReadUntilState,
}

impl<'a, R: AsyncBufRead + ?Sized + Unpin> Future for ReadUntilToken<'a, R> {
    type Output = Result<(usize, bool)>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        let mut rsz = 0;

        loop {
            let nsz = this.buf.len() - rsz;

            if let Some((pos, size)) = &mut this.state.consume_token {
                let sz = (*size - *pos).min(nsz);
                this.buf[rsz..rsz + sz].copy_from_slice(&this.token[*pos..*pos + sz]);
                *pos += sz;
                rsz += sz;
                if *pos == *size {
                    this.state.consume_token = None;
                }
                if rsz == this.buf.len() {
                    return Poll::Ready(Ok((rsz, false)));
                }
            } else {
                match Pin::new(&mut this.reader).poll_fill_buf(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
                    Poll::Ready(Ok(data)) if data.is_empty() => {
                        return Poll::Ready(Err(Error::from(ErrorKind::UnexpectedEof)))
                    }
                    Poll::Ready(Ok(data)) => {
                        let mut consume_size = data.len();
                        for (idx, b) in data.iter().enumerate() {
                            if *b == this.token[this.state.match_size] {
                                this.state.match_size += 1;
                                if this.state.match_size == this.token.len() {
                                    Pin::new(&mut this.reader).consume(idx + 1);
                                    this.state.match_size = 0;
                                    return Poll::Ready(Ok((rsz, true)));
                                }
                            } else if this.state.match_size > 0 {
                                this.state.consume_token = Some((0, this.state.match_size));
                                this.state.match_size = 0;
                                consume_size = idx;
                                break;
                            } else {
                                this.buf[rsz] = *b;
                                rsz += 1;
                                if rsz == this.buf.len() {
                                    Pin::new(&mut this.reader).consume(idx + 1);
                                    return Poll::Ready(Ok((rsz, false)));
                                }
                            }
                        }
                        Pin::new(&mut this.reader).consume(consume_size);
                    }
                }
            }
        }
    }
}

pub struct ExceptToken<'a, R: ?Sized> {
    reader: &'a mut R,
    token: &'a [u8],
    match_size: usize,
}

impl<'a, R: AsyncBufRead + ?Sized + Unpin> Future for ExceptToken<'a, R> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;

        loop {
            match Pin::new(&mut this.reader).poll_fill_buf(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
                Poll::Ready(Ok(data)) if data.is_empty() => {
                    return Poll::Ready(Err(Error::from(ErrorKind::UnexpectedEof)))
                }
                Poll::Ready(Ok(data)) => {
                    for b in data {
                        if *b == this.token[this.match_size] {
                            this.match_size += 1;
                            if this.match_size == this.token.len() {
                                Pin::new(&mut this.reader).consume(this.match_size);
                                return Poll::Ready(Ok(()));
                            }
                        } else {
                            return Poll::Ready(Err(Error::from(ErrorKind::InvalidData)));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::http::token_reader::{AsyncTokenReader, ReadUntilState};
    use futures::io::BufReader;

    #[async_std::test]
    async fn test_read_until_token() {
        let data: &[u8] = b"12AB567890ABC12345ABC6";
        let mut reader = BufReader::new(data);
        let mut buf = [0; 3];
        let mut state = ReadUntilState::default();

        let res = reader.read_until_token(b"ABC", &mut buf, &mut state).await;
        assert!(matches!(res, Ok((3, false))));
        assert_eq!(&buf, b"12A");

        let res = reader.read_until_token(b"ABC", &mut buf, &mut state).await;
        assert!(matches!(res, Ok((3, false))));
        assert_eq!(&buf, b"B56");

        let res = reader.read_until_token(b"ABC", &mut buf, &mut state).await;
        assert!(matches!(res, Ok((3, false))));
        assert_eq!(&buf, b"789");

        let res = reader.read_until_token(b"ABC", &mut buf, &mut state).await;
        assert!(matches!(res, Ok((1, true))));
        assert_eq!(&buf[..1], b"0");

        let res = reader.read_until_token(b"ABC", &mut buf, &mut state).await;
        assert!(matches!(res, Ok((3, false))));
        assert_eq!(&buf, b"123");

        let res = reader.read_until_token(b"ABC", &mut buf, &mut state).await;
        assert!(matches!(res, Ok((2, true))));
        assert_eq!(&buf[..2], b"45");

        let res = reader.read_until_token(b"ABC", &mut buf, &mut state).await;
        assert!(matches!(res, Err(_)));
    }

    #[async_std::test]
    async fn test_read_expect_token() {
        let data: &[u8] = b"ABCABC";
        let mut reader = BufReader::new(data);
        assert!(reader.except_token(b"ABC").await.is_ok());
        assert!(reader.except_token(b"ABC").await.is_ok());
        assert!(reader.except_token(b"ABC").await.is_err());
    }
}
