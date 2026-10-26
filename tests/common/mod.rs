// SPDX-License-Identifier: BSD-2-Clause
// Copyright (c) 2026 Faraz Vahedi <kfv@FreeBSD.org>

//! A one-shot HTTP/1.1 server on loopback for exercising the client.

// Each test crate uses a different subset.
#![allow(dead_code)]

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

/// What the server received.
pub struct Captured {
    pub method: String,
    pub target: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    /// Whether the whole body declared by `Content-Length` arrived.
    pub complete: bool,
}

impl Captured {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

/// How the server answers.
pub enum Reply {
    /// This status, with this JSON body.
    Json(u16, Vec<u8>),
    /// Nothing; wait until the client hangs up.
    Hang,
}

/// Accepts one request and answers with `status` and `body`.
pub async fn serve_once(
    status: u16,
    body: Vec<u8>,
) -> (String, JoinHandle<Captured>) {
    serve(Reply::Json(status, body)).await
}

/// Accepts one request and answers as `reply` says.  A client hanging up
/// early is not an error here: some tests make it do exactly that.
pub async fn serve(reply: Reply) -> (String, JoinHandle<Captured>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        let mut buf = Vec::new();
        let mut tmp = [0u8; 8192];
        let head_end = loop {
            if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                break i + 4;
            }
            let n = sock.read(&mut tmp).await.unwrap_or(0);
            if n == 0 {
                // The client gave up before finishing its headers.
                return Captured {
                    method: String::new(),
                    target: String::new(),
                    headers: Vec::new(),
                    body: buf,
                    complete: false,
                };
            }
            buf.extend_from_slice(&tmp[..n]);
        };
        let head = String::from_utf8(buf[..head_end].to_vec()).unwrap();
        let mut lines = head.split("\r\n");
        let mut start = lines.next().unwrap().split(' ');
        let method = start.next().unwrap().to_owned();
        let target = start.next().unwrap().to_owned();
        let headers: Vec<(String, String)> = lines
            .filter_map(|l| l.split_once(':'))
            .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
            .collect();
        let len = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
            .map_or(0, |(_, v)| v.parse::<usize>().unwrap());
        let mut body = buf[head_end..].to_vec();
        while body.len() < len {
            let n = sock.read(&mut tmp).await.unwrap_or(0);
            if n == 0 {
                break;
            }
            body.extend_from_slice(&tmp[..n]);
        }
        let complete = body.len() == len;
        match reply {
            Reply::Json(status, payload) => {
                let head = format!(
                    "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\n\
                     content-length: {}\r\nconnection: close\r\n\r\n",
                    payload.len()
                );
                // The client may stop reading part-way, e.g. at its size cap.
                let _ = sock.write_all(head.as_bytes()).await;
                let _ = sock.write_all(&payload).await;
                let _ = sock.shutdown().await;
            }
            Reply::Hang => while sock.read(&mut tmp).await.unwrap_or(0) > 0 {},
        }
        Captured {
            method,
            target,
            headers,
            body,
            complete,
        }
    });
    (format!("http://{addr}"), task)
}
