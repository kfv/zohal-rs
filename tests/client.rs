// SPDX-License-Identifier: BSD-2-Clause
// Copyright (c) 2026 Faraz Vahedi <kfv@FreeBSD.org>

//! Client behaviour the spec does not cover: transport, limits, headers and
//! uploads.

use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use serde_json::json;

use zohal::biometric::MediaType;
use zohal::identity::{CardImage, NationalCardOcrRequest, ShahkarRequest};
use zohal::{Client, Error, MAX_RESPONSE_BYTES, Upload};

mod common;

use common::{Reply, serve, serve_once};

const TOKEN: &str = "test-token";

fn client(base: &str) -> Client {
    Client::builder(TOKEN).base_url(base).build().unwrap()
}

fn shahkar() -> ShahkarRequest {
    ShahkarRequest {
        mobile: "09120000000".into(),
        national_code: "0012345678".into(),
    }
}

/// A successful Shahkar response padded to exactly `len` bytes.
fn matched_body(len: usize) -> Vec<u8> {
    let head = br#"{"response_body":{"data":{"matched":true},"message":""#;
    let tail = br#"","error_code":null},"result":1}"#;
    let mut body = head.to_vec();
    body.resize(len - tail.len(), b'x');
    body.extend_from_slice(tail);
    assert_eq!(body.len(), len);
    body
}

fn media_ok() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "response_body": {"type": "selfie_video", "id": "m-1"},
        "result": 1
    }))
    .unwrap()
}

/// The part of a multipart body from its `name="…"` header to the next
/// boundary.
fn part<'a>(body: &'a [u8], name: &str) -> &'a [u8] {
    let key = format!("form-data; name=\"{name}\"");
    let start = body
        .windows(key.len())
        .position(|w| w == key.as_bytes())
        .unwrap_or_else(|| panic!("no part {name}"));
    let rest = &body[start..];
    let end = rest
        .windows(4)
        .position(|w| w == b"\r\n--")
        .unwrap_or(rest.len());
    &rest[..end]
}

#[tokio::test]
async fn sends_accept_and_user_agent() {
    let (base, server) = serve_once(200, matched_body(100)).await;
    client(&base).shahkar(&shahkar()).await.unwrap();
    let got = server.await.unwrap();
    assert_eq!(got.header("accept"), Some("application/json"));
    let ua = format!("zohal/{}", env!("CARGO_PKG_VERSION"));
    assert_eq!(got.header("user-agent"), Some(ua.as_str()));
}

/// A body of exactly the cap is read; one byte more is refused.
#[tokio::test]
async fn response_size_is_capped() {
    let (base, server) =
        serve_once(200, matched_body(MAX_RESPONSE_BYTES)).await;
    let m = client(&base).shahkar(&shahkar()).await.unwrap();
    assert!(m.matched);
    server.await.unwrap();

    let (base, server) =
        serve_once(200, matched_body(MAX_RESPONSE_BYTES + 1)).await;
    let err = client(&base).shahkar(&shahkar()).await.unwrap_err();
    assert!(matches!(err, Error::ResponseTooLarge), "{err:?}");
    server.await.unwrap();
}

#[tokio::test]
async fn timeout_is_a_transport_error() {
    let (base, server) = serve(Reply::Hang).await;
    let c = Client::builder(TOKEN)
        .base_url(base)
        .timeout(Duration::from_millis(200))
        .build()
        .unwrap();
    match c.shahkar(&shahkar()).await {
        Err(Error::Transport(e)) => assert!(e.is_timeout(), "{e:?}"),
        other => panic!("expected a timeout, got {other:?}"),
    }
    server.await.unwrap();
}

#[tokio::test]
async fn connection_failure_is_a_transport_error() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let err = client(&base).shahkar(&shahkar()).await.unwrap_err();
    assert!(matches!(err, Error::Transport(_)), "{err:?}");
}

#[tokio::test]
async fn uses_the_given_http_client() {
    let mut headers = zohal::reqwest::header::HeaderMap::new();
    headers.insert("x-test", "yes".parse().unwrap());
    let http = zohal::reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();
    let (base, server) = serve_once(200, matched_body(100)).await;
    let c = Client::builder(TOKEN)
        .base_url(base)
        .http_client(http)
        .build()
        .unwrap();
    c.shahkar(&shahkar()).await.unwrap();
    let got = server.await.unwrap();
    assert_eq!(got.header("x-test"), Some("yes"));
    let auth = format!("Bearer {TOKEN}");
    assert_eq!(got.header("authorization"), Some(auth.as_str()));
}

/// Percent-encoding does not neutralise dot segments, so these never leave
/// the client.  Nothing listens at the base URL: a request that did go out
/// would fail as a transport error instead.
#[tokio::test]
async fn liveness_result_rejects_dot_segments() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let c = client(&base);
    for id in ["", ".", ".."] {
        let err = c.liveness_result(id).await.unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)), "{id:?}: {err:?}");
    }
}

/// A file upload is streamed with a known length: the request carries
/// `Content-Length`, not chunked encoding, and the bytes arrive intact.
#[tokio::test]
async fn file_upload_streams_with_length() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join("zohal-selfie.mp4");
    // Larger than any single read buffer, so it goes out in chunks.
    let content: Vec<u8> = (0..3u32 << 20).map(|i| (i % 251) as u8).collect();
    std::fs::write(&path, &content).unwrap();

    let (base, server) = serve_once(200, media_ok()).await;
    let upload = Upload::from_file(&path).await.unwrap();
    let media = client(&base)
        .upload_media(MediaType::SelfieVideo, upload)
        .await
        .unwrap();
    assert_eq!(media.id, "m-1");

    let got = server.await.unwrap();
    assert!(got.complete);
    assert!(got.header("transfer-encoding").is_none());
    let file = part(&got.body, "file");
    assert!(file.ends_with(&content), "file content not in the part");
    let head = String::from_utf8_lossy(&file[..file.len() - content.len()]);
    assert!(head.contains("filename=\"zohal-selfie.mp4\""), "{head}");
    assert!(head.contains("Content-Type: video/mp4"), "{head}");
}

#[tokio::test]
async fn file_upload_reports_missing_file() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let err = Upload::from_file(dir.join("no-such-file.mp4"))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Io(_)), "{err:?}");
}

#[tokio::test]
async fn file_upload_needs_a_file_name() {
    for path in ["/", ".."] {
        let err = Upload::from_file(path).await.unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)), "{path}: {err:?}");
    }
}

type Chunk = Result<&'static [u8], std::io::Error>;

/// A stream of fixed chunks, so tests control exactly where they split.
struct Chunks(VecDeque<Chunk>);

impl Chunks {
    fn new(chunks: impl IntoIterator<Item = Chunk>) -> Self {
        Chunks(chunks.into_iter().collect())
    }
}

impl futures_core::Stream for Chunks {
    type Item = Chunk;

    fn poll_next(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        Poll::Ready(self.get_mut().0.pop_front())
    }
}

/// Whether any error in the chain mentions `needle`.
fn chain_mentions(err: &Error, needle: &str) -> bool {
    let mut cur: Option<&dyn std::error::Error> = Some(err);
    while let Some(e) = cur {
        if e.to_string().contains(needle) {
            return true;
        }
        cur = e.source();
    }
    false
}

/// Sends `upload` and checks the request failed before a complete body
/// reached the server, with `needle` somewhere in the error chain.
async fn assert_upload_aborted(upload: Upload, needle: &str, case: &str) {
    let (base, server) = serve_once(200, media_ok()).await;
    let err = client(&base)
        .upload_media(MediaType::SelfieVideo, upload)
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Transport(_)), "{case}: {err:?}");
    assert!(chain_mentions(&err, needle), "{case}: {err:?}");
    let got = server.await.unwrap();
    assert!(!got.complete, "{case}: server got a complete request");
}

/// A stream upload goes out with the length given and, unless one is set,
/// no content type.
#[tokio::test]
async fn stream_upload_streams_with_length() {
    let stream = Chunks::new([Ok(&b"selfie "[..]), Ok(&b"bytes"[..])]);
    let upload = Upload::from_stream("v.mp4", stream, 12);
    let (base, server) = serve_once(200, media_ok()).await;
    client(&base)
        .upload_media(MediaType::SelfieVideo, upload)
        .await
        .unwrap();
    let got = server.await.unwrap();
    assert!(got.complete);
    assert!(got.header("transfer-encoding").is_none());
    let file = String::from_utf8_lossy(part(&got.body, "file")).into_owned();
    assert!(file.contains("filename=\"v.mp4\""), "{file}");
    assert!(file.ends_with("selfie bytes"), "{file}");
    assert!(!file.contains("Content-Type"), "{file}");
}

/// A stream that yields more or fewer bytes than declared, or fails,
/// aborts the request instead of sending a malformed body.
#[tokio::test]
async fn stream_upload_with_wrong_length_or_error_fails() {
    let cases: [(&str, Vec<Chunk>, u64, &str); 4] = [
        ("shorter", vec![Ok(&b"abc"[..])], 10, "shorter than"),
        ("longer", vec![Ok(&b"0123456789"[..])], 3, "longer than"),
        (
            "longer after filling",
            vec![Ok(&b"012"[..]), Ok(&b"3456789"[..])],
            3,
            "longer than",
        ),
        (
            "failing",
            vec![Ok(&b"abc"[..]), Err(std::io::Error::other("disk on fire"))],
            6,
            "disk on fire",
        ),
    ];
    for (case, chunks, len, needle) in cases {
        let upload = Upload::from_stream("v.mp4", Chunks::new(chunks), len);
        assert_upload_aborted(upload, needle, case).await;
    }
}

/// The length is taken when the upload is made, so a file that grows or
/// shrinks before it is sent fails the request.
#[tokio::test]
async fn file_upload_that_changes_size_fails() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    for (case, needle) in
        [("grows", "longer than"), ("shrinks", "shorter than")]
    {
        let path = dir.join(format!("zohal-{case}.mp4"));
        std::fs::write(&path, vec![b'x'; 1000]).unwrap();
        let upload = Upload::from_file(&path).await.unwrap();
        let f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        if case == "grows" {
            std::io::Write::write_all(&mut &f, b"more").unwrap();
        } else {
            f.set_len(10).unwrap();
        }
        assert_upload_aborted(upload, needle, case).await;
    }
}

#[tokio::test]
async fn file_upload_needs_a_regular_file() {
    let dir = env!("CARGO_TARGET_TMPDIR");
    let err = Upload::from_file(dir).await.unwrap_err();
    assert!(matches!(err, Error::InvalidArgument(_)), "{err:?}");
}

#[test]
fn upload_rejects_bad_content_type() {
    let err = Upload::from_bytes("x.bin", b"x".to_vec())
        .content_type("not a type")
        .unwrap_err();
    assert!(matches!(err, Error::InvalidArgument(_)), "{err:?}");
}

/// OCR sends an uploaded side as a file and a URL side as plain text.
#[tokio::test]
async fn ocr_sends_files_as_files_and_urls_as_text() {
    let ok = json!({
        "response_body": {
            "data": {"front": null, "back": null},
            "message": "",
            "error_code": null
        },
        "result": 1
    });
    let (base, server) =
        serve_once(200, serde_json::to_vec(&ok).unwrap()).await;
    let front = Upload::from_bytes("f.jpg", b"jpeg".to_vec())
        .content_type("image/jpeg")
        .unwrap();
    client(&base)
        .national_card_ocr(NationalCardOcrRequest {
            front: Some(CardImage::File(front)),
            back: Some(CardImage::Url("https://example.test/b.jpg".into())),
        })
        .await
        .unwrap();
    let got = server.await.unwrap();
    let front = String::from_utf8_lossy(part(&got.body, "national_card_front"))
        .into_owned();
    assert!(front.contains("filename=\"f.jpg\""), "{front}");
    assert!(front.contains("Content-Type: image/jpeg"), "{front}");
    assert!(front.ends_with("jpeg"), "{front}");
    let back = String::from_utf8_lossy(part(&got.body, "national_card_back"))
        .into_owned();
    assert!(!back.contains("filename="), "{back}");
    assert!(back.ends_with("https://example.test/b.jpg"), "{back}");
}
