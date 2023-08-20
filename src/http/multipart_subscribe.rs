use bytes::{BufMut, Bytes, BytesMut};
use futures_util::{stream::BoxStream, Stream, StreamExt};
use mime::Mime;

use crate::Response;

static PART_HEADER: Bytes =
    Bytes::from_static(b"--graphql\r\nContent-Type: application/json\r\n\r\n");
static EOF: Bytes = Bytes::from_static(b"--graphql--\r\n");
static CRLF: Bytes = Bytes::from_static(b"\r\n");
static HEARTBEAT: Bytes = Bytes::from_static(b"{}\r\n");

/// Create a stream for `multipart/mixed` responses.
///
/// Reference: <https://www.apollographql.com/docs/router/executing-operations/subscription-multipart-protocol/>
pub fn create_multipart_mixed_stream<'a>(
    input: impl Stream<Item = Response> + Send + Unpin + 'a,
    heartbeat_timer: impl Stream<Item = ()> + Send + Unpin + 'a,
) -> BoxStream<'a, Bytes> {
    let mut input = input.fuse();
    let mut heartbeat_timer = heartbeat_timer.fuse();

    async_stream::stream! {
        loop {
            futures_util::select! {
                item = input.next() => {
                    match item {
                        Some(resp) => {
                            let data = BytesMut::new();
                            let mut writer = data.writer();
                            if serde_json::to_writer(&mut writer, &resp).is_err() {
                                continue;
                            }

                            yield PART_HEADER.clone();
                            yield writer.into_inner().freeze();
                            yield CRLF.clone();
                        }
                        None => break,
                    }
                }
                _ = heartbeat_timer.next() => {
                    yield PART_HEADER.clone();
                    yield HEARTBEAT.clone();
                }
            }
        }

        yield EOF.clone();
    }
    .boxed()
}

fn parse_accept(accept: &str) -> Vec<Mime> {
    let mut items = accept
        .split(',')
        .map(str::trim)
        .filter_map(|item| {
            let mime: Mime = item.parse().ok()?;
            let q = mime
                .get_param("q")
                .and_then(|value| Some((value.as_str().parse::<f32>().ok()? * 1000.0) as i32))
                .unwrap_or(1000);
            Some((mime, q))
        })
        .collect::<Vec<_>>();
    items.sort_by(|(_, qa), (_, qb)| qb.cmp(qa));
    items.into_iter().map(|(mime, _)| mime).collect()
}

/// Check accept is multipart-mixed
///
/// # Example header
///
/// ```text
/// Accept: multipart/mixed; boundary="graphql"; subscriptionSpec="1.0"
/// ```
///
/// the value for boundary should always be `graphql`, and the value
/// for `subscriptionSpec` should always be `1.0`.
///
/// Reference: <https://www.apollographql.com/docs/router/executing-operations/subscription-multipart-protocol/>
pub fn is_accept_multipart_mixed(accept: &str) -> bool {
    for mime in parse_accept(accept) {
        if mime.type_() == mime::APPLICATION && mime.subtype() == mime::JSON {
            return false;
        }

        if mime.type_() == mime::MULTIPART
            && mime.subtype() == "mixed"
            && mime.get_param(mime::BOUNDARY).map(|value| value.as_str()) == Some("graphql")
            && mime
                .get_param("subscriptionSpec")
                .map(|value| value.as_str())
                == Some("1.0")
        {
            return true;
        }
    }

    false
}
