use std::{time::Duration, io::Write};

use reqwest::Client;

pub fn client() -> Client {
    Client::builder().no_proxy().build().unwrap()
}

pub async fn wait_server_ready() {
    async_std::task::sleep(Duration::from_secs(1)).await;
}

#[derive(Debug, Clone, Copy)]
pub enum ContentEncoding {
    Gzip,
    Deflate,
    Br,
    Zstd
}

impl ContentEncoding {
    pub const fn header(&self) -> &'static str {
        match self {
            ContentEncoding::Gzip => "gzip",
            ContentEncoding::Deflate => "deflate",
            ContentEncoding::Br => "br",
            ContentEncoding::Zstd => "zstd",
        }
    }

    pub const ALL: &'static [ContentEncoding] = &[
        ContentEncoding::Gzip,
        ContentEncoding::Deflate,
        ContentEncoding::Br,
        ContentEncoding::Zstd,
    ];
}

// #[cfg(feature = "compression")]
pub fn compress_query(data: impl AsRef<str>, algo: ContentEncoding) -> Vec<u8> {
    match algo {
        ContentEncoding::Gzip => {
            let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(data.as_ref().as_bytes()).unwrap();
            encoder.finish().unwrap()
        },
        ContentEncoding::Deflate => {
            let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(data.as_ref().as_bytes()).unwrap();
            encoder.finish().unwrap()
        },
        ContentEncoding::Br => {
            let mut buff = Vec::new();
            let mut encoder = brotli::CompressorWriter::with_params(&mut buff, 4096, &Default::default());
            encoder.write_all(data.as_ref().as_bytes()).unwrap();
            encoder.flush().unwrap();
            encoder.into_inner().to_vec()
        },
        ContentEncoding::Zstd => {
            let mut buff = Vec::new();
            let mut encoder = zstd::stream::Encoder::new(&mut buff, 9).unwrap();
            encoder.write_all(data.as_ref().as_bytes()).unwrap();
            encoder.finish().unwrap().to_vec()
        }
    }
}
