// SPDX-License-Identifier: BSD-2-Clause
// Copyright (c) 2026 Faraz Vahedi <kfv@FreeBSD.org>

use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures_core::Stream;
use reqwest::multipart::Part;

use crate::Error;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// A file sent as a multipart upload.
///
/// Files and streams are sent as they are read, not buffered in memory.
/// Every upload declares its length up front, so the request carries
/// `Content-Length` rather than chunked encoding, and the bytes actually
/// sent are counted: if they come to more or fewer than declared, the
/// request is aborted rather than sent malformed.  An upload is consumed by
/// the request that sends it.
#[derive(Debug)]
pub struct Upload {
    // Boxed: a Part is large, and Upload sits inside enums such as CardImage.
    part: Box<Part>,
}

impl Upload {
    fn new(part: Part) -> Self {
        Upload {
            part: Box::new(part),
        }
    }

    /// Uploads bytes already in memory.  No content type is sent unless
    /// set with [`content_type`](Self::content_type).
    pub fn from_bytes(
        file_name: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
    ) -> Self {
        Upload::new(Part::bytes(bytes.into()).file_name(file_name.into()))
    }

    /// Streams a file from disk.  The file name comes from the path, the
    /// length from the file's metadata, and the content type is guessed
    /// from the extension, falling back to `application/octet-stream`.  If
    /// the file changes size before it has been sent, the request fails.
    pub async fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let file_name = path
            .file_name()
            .ok_or(Error::InvalidArgument("upload path has no file name"))?
            .to_string_lossy()
            .into_owned();
        let file = tokio::fs::File::open(path).await.map_err(Error::Io)?;
        let meta = file.metadata().await.map_err(Error::Io)?;
        if !meta.is_file() {
            return Err(Error::InvalidArgument("upload path is not a file"));
        }
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        let stream = tokio_util::io::ReaderStream::new(file);
        let part = exact(stream, meta.len())
            .file_name(file_name)
            .mime_str(mime.as_ref())
            .map_err(|_| {
                Error::InvalidArgument("upload content type does not parse")
            })?;
        Ok(Upload::new(part))
    }

    /// Streams bytes from any source, such as a channel or a decoder.  It
    /// must yield exactly `len` bytes; if it yields more or fewer, or an
    /// error, the request fails.  No content type is sent unless set with
    /// [`content_type`](Self::content_type).
    pub fn from_stream<S, B, E>(
        file_name: impl Into<String>,
        stream: S,
        len: u64,
    ) -> Self
    where
        S: Stream<Item = Result<B, E>> + Send + 'static,
        B: Into<Bytes>,
        E: Into<BoxError>,
    {
        Upload::new(exact(stream, len).file_name(file_name.into()))
    }

    /// Sets the content type, e.g. `video/mp4` or `image/jpeg`.
    pub fn content_type(self, mime: &str) -> Result<Self, Error> {
        let part = (*self.part).mime_str(mime).map_err(|_| {
            Error::InvalidArgument("upload content type does not parse")
        })?;
        Ok(Upload::new(part))
    }

    pub(crate) fn into_part(self) -> Part {
        *self.part
    }
}

fn exact<S, B, E>(stream: S, len: u64) -> Part
where
    S: Stream<Item = Result<B, E>> + Send + 'static,
    B: Into<Bytes>,
    E: Into<BoxError>,
{
    let body = reqwest::Body::wrap_stream(Exact {
        inner: Box::pin(stream),
        left: len,
        done: false,
    });
    Part::stream_with_length(body, len)
}

/// Passes a stream through, failing it if it yields more or fewer bytes
/// than declared.  The request's `Content-Length` is computed from the
/// declared length, so without this a longer stream would be cut off at
/// that length and sent as a malformed body.
struct Exact<S> {
    inner: Pin<Box<S>>,
    left: u64,
    done: bool,
}

impl<S, B, E> Stream for Exact<S>
where
    S: Stream<Item = Result<B, E>>,
    B: Into<Bytes>,
    E: Into<BoxError>,
{
    type Item = Result<Bytes, BoxError>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // Unpin: the inner stream is behind its own Pin<Box>.
        let this = self.get_mut();
        if this.done {
            return Poll::Ready(None);
        }
        let item = match this.inner.as_mut().poll_next(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(item) => item,
        };
        let failure: BoxError = match item {
            Some(Ok(chunk)) => {
                let chunk: Bytes = chunk.into();
                // usize is at most 64 bits on every Rust target.
                match this.left.checked_sub(chunk.len() as u64) {
                    Some(left) => {
                        this.left = left;
                        return Poll::Ready(Some(Ok(chunk)));
                    }
                    None => "upload is longer than its declared length".into(),
                }
            }
            Some(Err(e)) => e.into(),
            None if this.left == 0 => {
                this.done = true;
                return Poll::Ready(None);
            }
            None => "upload is shorter than its declared length".into(),
        };
        this.done = true;
        Poll::Ready(Some(Err(failure)))
    }
}
