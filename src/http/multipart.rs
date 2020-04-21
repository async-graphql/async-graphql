use super::token_reader::*;
use crate::ParseRequestError;
use futures::io::BufReader;
use futures::{AsyncBufRead, AsyncRead};
use http::{header::HeaderName, HeaderMap, HeaderValue};
use itertools::Itertools;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tempdir::TempDir;

const MAX_HEADERS: usize = 16;

pub enum PartData {
    Bytes(Vec<u8>),
    File(PathBuf),
}

pub struct Part {
    pub name: Option<String>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub size: usize,
    pub data: PartData,
}

impl Part {
    pub fn create_reader<'a>(&'a self) -> Result<Box<dyn Read + 'a>, std::io::Error> {
        let reader: Box<dyn Read> = match &self.data {
            PartData::Bytes(bytes) => Box::new(Cursor::new(bytes)),
            PartData::File(path) => Box::new(File::open(path)?),
        };
        Ok(reader)
    }
}

struct ContentDisposition {
    name: Option<String>,
    filename: Option<String>,
}

impl ContentDisposition {
    fn parse(value: &str) -> Result<ContentDisposition, ParseRequestError> {
        let name = regex::Regex::new("name=\"(?P<name>.*?)\"")
            .unwrap()
            .captures(value)
            .and_then(|caps| caps.name("name").map(|m| m.as_str().to_string()));
        let filename = regex::Regex::new("filename=\"(?P<filename>.*?)\"")
            .unwrap()
            .captures(value)
            .and_then(|caps| caps.name("filename").map(|m| m.as_str().to_string()));
        Ok(ContentDisposition { name, filename })
    }
}

pub struct Multipart {
    pub temp_dir: Option<TempDir>,
    pub parts: Vec<Part>,
}

impl Multipart {
    pub async fn parse<R: AsyncRead + Unpin>(
        reader: R,
        boundary: &str,
        temp_dir_in: Option<&Path>,
        max_file_size: Option<usize>,
        max_num_files: Option<usize>,
    ) -> Result<Multipart, ParseRequestError> {
        let mut reader = BufReader::new(reader);
        let mut temp_dir = None;
        let mut parts = Vec::new();
        let boundary = format!("--{}", boundary);
        let max_num_files = max_num_files.unwrap_or(std::usize::MAX);
        let max_file_size = max_file_size.unwrap_or(std::usize::MAX);
        let mut current_num_files = 0;

        // first part
        reader.except_token(boundary.as_bytes()).await?;
        reader.except_token(b"\r\n").await?;
        let headers = Self::parse_headers(&mut reader).await?;
        parts.push(
            Self::parse_body(
                &mut reader,
                &headers,
                &mut temp_dir,
                temp_dir_in,
                max_file_size,
                &boundary,
            )
            .await?,
        );
        Multipart::check_max_num_files(&mut parts, max_num_files, &mut current_num_files)?;

        // next parts
        loop {
            if reader.except_token(b"\r\n").await.is_err() {
                reader.except_token(b"--\r\n").await?;
                break;
            }

            let headers = Self::parse_headers(&mut reader).await?;
            parts.push(
                Self::parse_body(
                    &mut reader,
                    &headers,
                    &mut temp_dir,
                    temp_dir_in,
                    max_file_size,
                    &boundary,
                )
                .await?,
            );
            Multipart::check_max_num_files(&mut parts, max_num_files, &mut current_num_files)?;
        }

        Ok(Multipart { temp_dir, parts })
    }

    fn check_max_num_files(
        parts: &mut Vec<Part>,
        max_num_files: usize,
        current_num_files: &mut usize,
    ) -> Result<(), ParseRequestError> {
        if parts.last().unwrap().filename.is_some() {
            *current_num_files += 1;
            if *current_num_files > max_num_files {
                return Err(ParseRequestError::TooManyFiles);
            }
        }
        Ok(())
    }

    async fn parse_headers<R: AsyncBufRead + Unpin>(
        mut reader: R,
    ) -> Result<HeaderMap, ParseRequestError> {
        let mut buf = [0; 256];
        let mut header_data = Vec::new();
        let mut state = ReadUntilState::default();

        loop {
            let (size, found) = reader
                .read_until_token(b"\r\n\r\n", &mut buf, &mut state)
                .await?;
            header_data.extend_from_slice(&buf[..size]);
            if found {
                break;
            }
        }

        let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
        header_data.extend_from_slice(b"\r\n\r\n");
        let headers = match httparse::parse_headers(&header_data, &mut headers)
            .map_err(|_| ParseRequestError::InvalidMultipart)?
        {
            httparse::Status::Complete((_, headers)) => headers,
            _ => return Err(ParseRequestError::InvalidMultipart),
        };

        let mut headers_map = HeaderMap::new();
        for httparse::Header { name, value } in headers {
            headers_map.insert(
                HeaderName::from_str(name).map_err(|_| ParseRequestError::InvalidMultipart)?,
                HeaderValue::from_bytes(value).map_err(|_| ParseRequestError::InvalidMultipart)?,
            );
        }

        Ok(headers_map)
    }

    async fn parse_body<R: AsyncBufRead + Unpin>(
        mut reader: R,
        headers: &HeaderMap,
        temp_dir: &mut Option<TempDir>,
        temp_dir_in: Option<&Path>,
        max_file_size: usize,
        boundary: &str,
    ) -> Result<Part, ParseRequestError> {
        let content_disposition = headers
            .get(http::header::CONTENT_DISPOSITION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| ContentDisposition::parse(value).ok())
            .unwrap_or_else(|| ContentDisposition {
                name: None,
                filename: None,
            });
        let content_type = headers
            .get(http::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string);

        let mut buf = [0; 4096];
        let mut state = ReadUntilState::default();
        let mut total_size = 0;

        let part_data = if let Some(filename) = &content_disposition.filename {
            if temp_dir.is_none() {
                if let Some(temp_dir_in) = temp_dir_in {
                    *temp_dir = Some(TempDir::new_in(temp_dir_in, "async-graphql")?);
                } else {
                    *temp_dir = Some(TempDir::new("async-graphql")?);
                }
            }
            let temp_dir = temp_dir.as_mut().unwrap();
            let path = temp_dir.path().join(filename);
            let mut file = File::create(&path)?;

            loop {
                let (size, found) = reader
                    .read_until_token(boundary.as_bytes(), &mut buf, &mut state)
                    .await?;
                total_size += size;
                if total_size > max_file_size {
                    return Err(ParseRequestError::TooLarge);
                }
                file.write_all(&buf[..size])?;
                if found {
                    break;
                }
            }
            PartData::File(path)
        } else {
            let mut body = Vec::new();

            loop {
                let (size, found) = reader
                    .read_until_token(boundary.as_bytes(), &mut buf, &mut state)
                    .await?;
                total_size += size;
                body.extend_from_slice(&buf[..size]);
                if found {
                    break;
                }
            }
            PartData::Bytes(body)
        };

        Ok(Part {
            name: content_disposition.name,
            filename: content_disposition.filename,
            content_type,
            size: total_size,
            data: part_data,
        })
    }

    pub fn remove(&mut self, name: &str) -> Option<Part> {
        if let Some((pos, _)) = self.parts.iter().find_position(|part| {
            if let Some(part_name) = &part.name {
                part_name == name
            } else {
                false
            }
        }) {
            Some(self.parts.remove(pos))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn test_parse() {
        let data: &[u8] = b"--abbc761f78ff4d7cb7573b5a23f96ef0\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"fn.txt\"\r\n\
             Content-Type: text/plain; charset=utf-8\r\n\r\n\
             test\
             --abbc761f78ff4d7cb7573b5a23f96ef0\r\n\
             Content-Type: text/plain; charset=utf-8\r\n\r\n\
             data\
             --abbc761f78ff4d7cb7573b5a23f96ef0--\r\n";
        let multipart =
            Multipart::parse(data, "abbc761f78ff4d7cb7573b5a23f96ef0", None, None, None)
                .await
                .unwrap();
        assert_eq!(multipart.parts.len(), 2);

        let part_1 = &multipart.parts[0];
        assert_eq!(part_1.name.as_deref(), Some("file"));
        assert_eq!(part_1.filename.as_deref(), Some("fn.txt"));
        assert_eq!(
            part_1.content_type.as_deref(),
            Some("text/plain; charset=utf-8")
        );

        let part_2 = &multipart.parts[1];
        assert!(part_2.name.is_none());
        assert!(part_2.filename.is_none());
        assert_eq!(
            part_2.content_type.as_deref(),
            Some("text/plain; charset=utf-8")
        );
    }

    #[async_std::test]
    async fn test_parse_limit_file_size() {
        let data: &[u8] = b"--abbc761f78ff4d7cb7573b5a23f96ef0\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"fn.txt\"\r\n\
             Content-Type: text/plain; charset=utf-8\r\n\r\n\
             12345\
             --abbc761f78ff4d7cb7573b5a23f96ef0\r\n\
             Content-Type: text/plain; charset=utf-8\r\n\r\n\
             data\
             --abbc761f78ff4d7cb7573b5a23f96ef0--\r\n";

        assert!(Multipart::parse(
            data,
            "abbc761f78ff4d7cb7573b5a23f96ef0",
            None,
            Some(5),
            None,
        )
        .await
        .is_ok());

        assert!(Multipart::parse(
            data,
            "abbc761f78ff4d7cb7573b5a23f96ef0",
            None,
            Some(6),
            None,
        )
        .await
        .is_ok());

        assert!(Multipart::parse(
            data,
            "abbc761f78ff4d7cb7573b5a23f96ef0",
            None,
            Some(4),
            None,
        )
        .await
        .is_err());
    }

    #[async_std::test]
    async fn test_parse_limit_num_files() {
        let data: &[u8] = b"--abbc761f78ff4d7cb7573b5a23f96ef0\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"fn.txt\"\r\n\
             Content-Type: text/plain; charset=utf-8\r\n\r\n\
             12345\
             --abbc761f78ff4d7cb7573b5a23f96ef0\r\n\
             Content-Disposition: form-data; name=\"file1\"; filename=\"fn1.txt\"\r\n\r\n\
             data\
             --abbc761f78ff4d7cb7573b5a23f96ef0\r\n\
             Content-Disposition: form-data; name=\"file2\"\r\n\r\n\
             data\
             --abbc761f78ff4d7cb7573b5a23f96ef0--\r\n";

        assert!(Multipart::parse(
            data,
            "abbc761f78ff4d7cb7573b5a23f96ef0",
            None,
            None,
            Some(1)
        )
        .await
        .is_err());

        assert!(Multipart::parse(
            data,
            "abbc761f78ff4d7cb7573b5a23f96ef0",
            None,
            None,
            Some(2)
        )
        .await
        .is_ok());

        assert!(Multipart::parse(
            data,
            "abbc761f78ff4d7cb7573b5a23f96ef0",
            None,
            None,
            Some(3)
        )
        .await
        .is_ok());
    }
}
