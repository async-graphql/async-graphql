use crate::http::GQLResponse;
use crate::{QueryResponse, Result};
use bytes::{buf::BufExt, Buf, Bytes};
use futures::{Stream, StreamExt};

/// Create a multipart response data stream.
pub fn multipart_stream(s: impl Stream<Item = Result<QueryResponse>>) -> impl Stream<Item = Bytes> {
    s.map(|res| serde_json::to_vec(&GQLResponse(res)).unwrap())
        .map(|data| {
            Bytes::from(format!(
                "\r\n---\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
                data.len()
            ))
            .chain(Bytes::from(data))
            .to_bytes()
        })
        .chain(futures::stream::once(async move {
            Bytes::from_static(b"\r\n-----\r\n")
        }))
}
