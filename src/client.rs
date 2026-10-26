// SPDX-License-Identifier: BSD-2-Clause
// Copyright (c) 2026 Faraz Vahedi <kfv@FreeBSD.org>

use std::fmt;
use std::net::IpAddr;
use std::time::Duration;

use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderValue};
use reqwest::{RequestBuilder, Url};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::{ApiError, Error, ResultCode};

/// Zohal's production endpoint.
pub const DEFAULT_BASE_URL: &str = "https://service.zohal.io";

/// Whole-request timeout used when the client builds its own HTTP client.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Connect timeout used when the client builds its own HTTP client.
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Upper bound on a response body.  The largest documented payload is the
/// national card OCR result with a Base64 face photo, which is well under
/// this.
pub const MAX_RESPONSE_BYTES: usize = 8 << 20;

const USER_AGENT: &str = concat!("zohal/", env!("CARGO_PKG_VERSION"));

/// A Zohal API client.
///
/// Cheap to clone; clones share the connection pool.
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    base: Url,
    auth: HeaderValue,
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.base.as_str())
            .finish_non_exhaustive()
    }
}

impl Client {
    /// Creates a client for the production endpoint with default settings.
    pub fn new(token: impl Into<String>) -> Result<Self, Error> {
        Self::builder(token).build()
    }

    /// Starts configuring a client.
    pub fn builder(token: impl Into<String>) -> ClientBuilder {
        ClientBuilder {
            token: token.into(),
            base_url: DEFAULT_BASE_URL.to_owned(),
            timeout: DEFAULT_TIMEOUT,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            http: None,
        }
    }

    /// The base URL requests are sent to.
    pub fn base_url(&self) -> &str {
        self.base.as_str()
    }

    pub(crate) async fn post_json<B, T>(
        &self,
        path: &'static str,
        body: &B,
        shape: Shape,
    ) -> Result<T, Error>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let url = self.url(split(path))?;
        self.execute(self.http.post(url).json(body), shape).await
    }

    pub(crate) async fn post_multipart<T>(
        &self,
        path: &'static str,
        form: reqwest::multipart::Form,
        shape: Shape,
    ) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let url = self.url(split(path))?;
        self.execute(self.http.post(url).multipart(form), shape)
            .await
    }

    pub(crate) async fn get<'a, T>(
        &self,
        segments: impl IntoIterator<Item = &'a str>,
        shape: Shape,
    ) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let url = self.url(segments)?;
        self.execute(self.http.get(url), shape).await
    }

    /// Appends path segments to the base URL, percent-encoding each one.
    /// An empty final segment yields a trailing slash, which some Zohal
    /// endpoints require.
    fn url<'a>(
        &self,
        segments: impl IntoIterator<Item = &'a str>,
    ) -> Result<Url, Error> {
        let mut url = self.base.clone();
        // Cannot fail: build() rejects cannot-be-a-base URLs.
        url.path_segments_mut()
            .map_err(|()| {
                Error::InvalidArgument("base URL cannot have a path")
            })?
            .pop_if_empty()
            .extend(segments);
        Ok(url)
    }

    async fn execute<T>(
        &self,
        req: RequestBuilder,
        shape: Shape,
    ) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let mut resp = req
            .header(AUTHORIZATION, self.auth.clone())
            .header(ACCEPT, "application/json")
            .send()
            .await
            .map_err(Error::Transport)?;
        let status = resp.status().as_u16();
        let mut body = Vec::new();
        while let Some(chunk) = resp.chunk().await.map_err(Error::Transport)? {
            if body.len() + chunk.len() > MAX_RESPONSE_BYTES {
                return Err(Error::ResponseTooLarge);
            }
            body.extend_from_slice(&chunk);
        }
        decode(status, &body, shape)
    }
}

/// Configures a [`Client`].
pub struct ClientBuilder {
    token: String,
    base_url: String,
    timeout: Duration,
    connect_timeout: Duration,
    http: Option<reqwest::Client>,
}

impl fmt::Debug for ClientBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientBuilder")
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .field("connect_timeout", &self.connect_timeout)
            .field("http", &self.http)
            .finish_non_exhaustive()
    }
}

impl ClientBuilder {
    /// Sends requests somewhere other than [`DEFAULT_BASE_URL`].  Must be
    /// `https`, except for loopback hosts.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Whole-request timeout.  Ignored if [`http_client`](Self::http_client)
    /// is set.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Connect timeout.  Ignored if [`http_client`](Self::http_client) is
    /// set.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Uses a preconfigured HTTP client, e.g. to set a proxy or share a
    /// connection pool.  Its timeouts apply instead of this builder's.
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http = Some(client);
        self
    }

    /// Validates the settings and creates the client.
    pub fn build(self) -> Result<Client, Error> {
        let base = Url::parse(&self.base_url)
            .map_err(|_| Error::InvalidArgument("base URL does not parse"))?;
        if base.cannot_be_a_base()
            || base.query().is_some()
            || base.fragment().is_some()
        {
            return Err(Error::InvalidArgument(
                "base URL must be a plain scheme://host[/path]",
            ));
        }
        // The token travels in a header on every request.
        match base.scheme() {
            "https" => {}
            "http" if is_loopback(&base) => {}
            _ => return Err(Error::InvalidArgument("base URL must use https")),
        }
        if self.token.is_empty() {
            return Err(Error::InvalidArgument("token is empty"));
        }
        let mut auth = HeaderValue::try_from(format!("Bearer {}", self.token))
            .map_err(|_| {
                Error::InvalidArgument("token is not a valid header value")
            })?;
        auth.set_sensitive(true);
        let http = match self.http {
            Some(http) => http,
            None => reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .timeout(self.timeout)
                .connect_timeout(self.connect_timeout)
                .build()
                .map_err(Error::Transport)?,
        };
        Ok(Client { http, base, auth })
    }
}

fn is_loopback(url: &Url) -> bool {
    match url.host_str() {
        Some("localhost") => true,
        Some(h) => h
            .trim_start_matches('[')
            .trim_end_matches(']')
            .parse::<IpAddr>()
            .is_ok_and(|ip| ip.is_loopback()),
        None => false,
    }
}

fn split(path: &'static str) -> impl Iterator<Item = &'static str> {
    path.strip_prefix('/').unwrap_or(path).split('/')
}

/// Where the payload sits in a successful response.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Shape {
    /// `{"response_body": {"data": T, "message", "error_code"}, "result"}`
    Data,
    /// `{"response_body": T, "result"}`, used by the biometric endpoints.
    Bare,
}

#[derive(serde::Deserialize)]
struct Envelope {
    // A crash path at Zohal emits the misspelt key.
    #[serde(alias = "reponse_body")]
    response_body: Option<Value>,
    result: Option<i64>,
}

/// Classifies a response.  Success requires all of a 2xx status, `result`
/// 1, and no `error_code` (or `error`, on the crash path); anything else is
/// an [`ApiError`].  Only then is the payload decoded.
pub(crate) fn decode<T: DeserializeOwned>(
    status: u16,
    body: &[u8],
    shape: Shape,
) -> Result<T, Error> {
    let ok_status = (200..300).contains(&status);
    let env: Envelope = match serde_json::from_slice(body) {
        Ok(env) => env,
        Err(source) if ok_status => {
            return Err(Error::Decode { status, source });
        }
        Err(_) => {
            return Err(Error::Api(ApiError {
                status,
                result: None,
                code: None,
                message: None,
            }));
        }
    };
    let mut rb = env.response_body.unwrap_or(Value::Null);
    let code = ["error_code", "error"]
        .iter()
        .find_map(|k| rb.get(k).filter(|v| !v.is_null()))
        .map(|v| match v {
            Value::String(s) => s.clone(),
            v => v.to_string(),
        });
    if !ok_status || env.result != Some(1) || code.is_some() {
        return Err(Error::Api(ApiError {
            status,
            result: env.result.map(ResultCode::from_code),
            code,
            message: rb
                .get("message")
                .and_then(Value::as_str)
                .map(str::to_owned),
        }));
    }
    let payload = match shape {
        Shape::Data => {
            rb.get_mut("data").map(Value::take).unwrap_or(Value::Null)
        }
        Shape::Bare => rb,
    };
    serde_json::from_value(payload)
        .map_err(|source| Error::Decode { status, source })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Matched;

    fn api(r: Result<Matched, Error>) -> ApiError {
        match r {
            Err(Error::Api(e)) => e,
            other => panic!("expected Api error, got {other:?}"),
        }
    }

    #[test]
    fn data_success() {
        let body = br#"{"response_body":{"data":{"matched":false},"message":"x","error_code":null},"result":1}"#;
        let m: Matched = decode(200, body, Shape::Data).unwrap();
        assert!(!m.matched);
    }

    #[test]
    fn error_code_wins_over_result_and_status() {
        // As documented for `card-inquiry`: HTTP 200, result 1, yet a
        // failure.
        let body = br#"{"response_body":{"data":null,"message":"m","error_code":"CARD_NOT_FOUND"},"result":1}"#;
        let e = api(decode(200, body, Shape::Data));
        assert_eq!(e.status, 200);
        assert_eq!(e.result, Some(ResultCode::Success));
        assert_eq!(e.code.as_deref(), Some("CARD_NOT_FOUND"));
        assert_eq!(e.message.as_deref(), Some("m"));
    }

    #[test]
    fn misspelt_crash_envelope() {
        let body = r#"{"reponse_body": {"data": null, "message": "نا موفق", "error": "Internal Server Error"}, "result": 5}"#;
        let e = api(decode(500, body.as_bytes(), Shape::Data));
        assert_eq!(e.result, Some(ResultCode::InternalError));
        assert_eq!(e.code.as_deref(), Some("Internal Server Error"));
        assert_eq!(e.message.as_deref(), Some("نا موفق"));
    }

    #[test]
    fn non_json() {
        let e = api(decode(502, b"<html>bad gateway</html>", Shape::Data));
        assert_eq!(
            (e.status, e.result, e.code, e.message),
            (502, None, None, None)
        );
        assert!(matches!(
            decode::<Matched>(200, b"<html>", Shape::Data),
            Err(Error::Decode { status: 200, .. })
        ));
    }

    #[test]
    fn empty_data_on_success_is_a_decode_error() {
        let body = br#"{"response_body":{"data":{},"message":"x","error_code":null},"result":1}"#;
        assert!(matches!(
            decode::<Matched>(200, body, Shape::Data),
            Err(Error::Decode { .. })
        ));
    }

    #[test]
    fn bare_shape() {
        let body = br#"{"response_body":{"matched":true},"result":1}"#;
        let m: Matched = decode(200, body, Shape::Bare).unwrap();
        assert!(m.matched);
    }

    fn classify(
        status: u16,
        result: Option<i64>,
        code: &Option<Value>,
        error: &Option<Value>,
        shape: Shape,
    ) {
        let mut rb = serde_json::json!({"matched": true, "message": "m"});
        if matches!(shape, Shape::Data) {
            rb = serde_json::json!({"data": rb, "message": "m"});
        }
        if let Some(c) = code {
            rb["error_code"] = c.clone();
        }
        if let Some(e) = error {
            rb["error"] = e.clone();
        }
        let mut env = serde_json::json!({"response_body": rb});
        if let Some(r) = result {
            env["result"] = r.into();
        }
        let body = serde_json::to_vec(&env).unwrap();
        let want = (200..300).contains(&status)
            && result == Some(1)
            && code.as_ref().is_none_or(Value::is_null)
            && error.as_ref().is_none_or(Value::is_null);
        let got = decode::<Matched>(status, &body, shape);
        assert_eq!(got.is_ok(), want, "{env} at HTTP {status}: {got:?}");
        if let Err(Error::Api(e)) = got {
            assert_eq!(e.status, status);
            assert_eq!(e.result.map(ResultCode::code), result);
        }
    }

    /// Every combination of the three failure signals: only the all-clear
    /// one may succeed.
    #[test]
    fn classification_is_exhaustive() {
        let statuses = [200, 201, 299, 300, 400, 401, 404, 429, 500, 502, 503];
        let results =
            [None, Some(1), Some(0), Some(4), Some(5), Some(6), Some(7)];
        let codes = [
            None,
            Some(Value::Null),
            Some(Value::from("X")),
            Some(Value::from(3)),
            Some(Value::from("")),
        ];
        let errors = [None, Some(Value::Null), Some(Value::from("E"))];
        for status in statuses {
            for result in results {
                for code in &codes {
                    for error in &errors {
                        classify(status, result, code, error, Shape::Data);
                        classify(status, result, code, error, Shape::Bare);
                    }
                }
            }
        }
    }

    #[test]
    fn builder_checks() {
        let bad = |url: &str| {
            matches!(
                Client::builder("t").base_url(url).build(),
                Err(Error::InvalidArgument(_))
            )
        };
        assert!(bad("http://service.zohal.io"));
        assert!(bad("ftp://service.zohal.io"));
        assert!(bad("https://service.zohal.io/?x=1"));
        assert!(bad("mailto:x@y"));
        assert!(bad("not a url"));
        for ok in [
            "http://127.0.0.1:8080",
            "http://[::1]:1",
            "http://localhost",
        ] {
            assert!(Client::builder("t").base_url(ok).build().is_ok(), "{ok}");
        }
        assert!(matches!(Client::new(""), Err(Error::InvalidArgument(_))));
        assert!(matches!(
            Client::new("a\nb"),
            Err(Error::InvalidArgument(_))
        ));
    }

    #[test]
    fn paths() {
        let c = Client::builder("t")
            .base_url("https://example.test/proxy/")
            .build()
            .unwrap();
        let u = c.url(split("/api/v0/services/biometric/media/")).unwrap();
        assert_eq!(
            u.as_str(),
            "https://example.test/proxy/api/v0/services/biometric/media/"
        );
        let u = c.url(["s", "a/b?c", ""]).unwrap();
        assert_eq!(u.as_str(), "https://example.test/proxy/s/a%2Fb%3Fc/");
    }

    #[test]
    fn token_is_not_printed() {
        let secret = "s3cr3t-t0ken";
        let b = Client::builder(secret);
        assert!(!format!("{b:?}").contains(secret));
        let c = b.build().unwrap();
        assert!(!format!("{c:?}").contains(secret));
    }
}
