// SPDX-License-Identifier: BSD-2-Clause
// Copyright (c) 2026 Faraz Vahedi <kfv@FreeBSD.org>

use std::fmt;

/// Errors returned by [`Client`](crate::Client).
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Zohal answered, but not with a successful result.
    Api(ApiError),
    /// The request never got a complete HTTP response: connection, TLS,
    /// timeout, an [`Upload`](crate::Upload) that failed or did not match
    /// its declared length, or similar.
    Transport(reqwest::Error),
    /// Zohal reported success but the body did not match the documented
    /// shape.
    Decode {
        /// HTTP status of the response.
        status: u16,
        /// What failed to parse.
        source: serde_json::Error,
    },
    /// The response body was larger than
    /// [`MAX_RESPONSE_BYTES`](crate::MAX_RESPONSE_BYTES).
    ResponseTooLarge,
    /// A client setting or argument was rejected before anything was sent.
    InvalidArgument(&'static str),
    /// A file for an [`Upload`](crate::Upload) could not be opened.
    Io(std::io::Error),
}

// A cause is either returned by source() or written by Display, never both,
// or reporters that walk the chain print it twice.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Api(e) => e.fmt(f),
            Error::Transport(_) => f.write_str("zohal request failed"),
            Error::Decode { status, .. } => {
                write!(f, "cannot decode zohal response (HTTP {status})")
            }
            Error::ResponseTooLarge => write!(
                f,
                "zohal response exceeds {} bytes",
                crate::MAX_RESPONSE_BYTES
            ),
            Error::InvalidArgument(what) => {
                write!(f, "invalid zohal client argument: {what}")
            }
            Error::Io(_) => f.write_str("cannot open upload"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Transport(e) => Some(e),
            Error::Decode { source, .. } => Some(source),
            Error::Io(e) => Some(e),
            Error::Api(_)
            | Error::ResponseTooLarge
            | Error::InvalidArgument(_) => None,
        }
    }
}

/// An unsuccessful answer from Zohal.
///
/// Zohal reports failure in three places, and they do not always agree: the
/// HTTP status, the envelope's `result`, and `response_body.error_code`.  Its
/// own documentation shows business failures such as `CARD_NOT_FOUND` with
/// HTTP 200 and `result` 1, so all three are kept here.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ApiError {
    /// HTTP status of the response.
    pub status: u16,
    /// The envelope's `result`, if the body had one.
    pub result: Option<ResultCode>,
    /// Machine-readable error code, e.g. `CARD_NOT_FOUND`, `min_length` or
    /// `not_authenticated`.
    pub code: Option<String>,
    /// Human-readable message, usually in Persian.  Validation messages are
    /// prefixed with the offending field name.
    pub message: Option<String>,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "zohal API error: HTTP {}", self.status)?;
        if let Some(r) = self.result {
            write!(f, ", result {}", r.code())?;
        }
        if let Some(c) = &self.code {
            write!(f, ", {c}")?;
        }
        if let Some(m) = &self.message {
            write!(f, ": {m}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ApiError {}

/// The envelope's `result` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ResultCode {
    /// `1`: the request was processed.  Not by itself proof of success; see
    /// [`ApiError`].
    Success,
    /// `4`: invalid request.
    InvalidRequest,
    /// `5`: internal error at Zohal.
    InternalError,
    /// `6`: invalid input, or the upstream service is unavailable.
    InvalidInput,
    /// Any other value.
    Other(i64),
}

impl ResultCode {
    /// Maps a raw `result` value.
    pub fn from_code(code: i64) -> Self {
        match code {
            1 => ResultCode::Success,
            4 => ResultCode::InvalidRequest,
            5 => ResultCode::InternalError,
            6 => ResultCode::InvalidInput,
            n => ResultCode::Other(n),
        }
    }

    /// The raw `result` value.
    pub fn code(self) -> i64 {
        match self {
            ResultCode::Success => 1,
            ResultCode::InvalidRequest => 4,
            ResultCode::InternalError => 5,
            ResultCode::InvalidInput => 6,
            ResultCode::Other(n) => n,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    /// Walks the chain like an error reporter and checks no level repeats
    /// its cause's text.
    fn assert_no_repeats(e: &Error) {
        let mut cur: &dyn std::error::Error = e;
        while let Some(next) = cur.source() {
            let (outer, inner) = (cur.to_string(), next.to_string());
            assert!(!outer.contains(&inner), "{outer:?} repeats {inner:?}");
            cur = next;
        }
    }

    #[test]
    fn chain_has_no_repeats() {
        let source = serde_json::from_str::<u8>("x").unwrap_err();
        let decode = Error::Decode {
            status: 200,
            source,
        };
        assert!(decode.source().is_some());
        assert_no_repeats(&decode);

        let api = Error::Api(ApiError {
            status: 400,
            result: Some(ResultCode::InvalidInput),
            code: Some("min_length".into()),
            message: Some("mobile: too short".into()),
        });
        assert!(api.source().is_none());
        assert_eq!(
            api.to_string(),
            "zohal API error: HTTP 400, result 6, min_length: mobile: too short"
        );

        let io = Error::Io(std::io::Error::other("disk on fire"));
        assert!(io.source().is_some());
        assert_no_repeats(&io);
    }

    #[test]
    fn result_code_round_trips() {
        for n in [i64::MIN, -1, 0, 1, 2, 4, 5, 6, 7, i64::MAX] {
            assert_eq!(ResultCode::from_code(n).code(), n);
        }
        assert_eq!(ResultCode::from_code(1), ResultCode::Success);
        assert_eq!(ResultCode::from_code(4), ResultCode::InvalidRequest);
        assert_eq!(ResultCode::from_code(5), ResultCode::InternalError);
        assert_eq!(ResultCode::from_code(6), ResultCode::InvalidInput);
        assert_eq!(ResultCode::from_code(7), ResultCode::Other(7));
    }
}
