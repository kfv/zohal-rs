// SPDX-License-Identifier: BSD-2-Clause
// Copyright (c) 2026 Faraz Vahedi <kfv@FreeBSD.org>

//! Unofficial client for the [Zohal](https://zohal.io) enquiry API:
//! identity verification, banking, and more general enquiries.  Not
//! affiliated with or endorsed by Zohal.
//!
//! ```no_run
//! # async fn f() -> Result<(), zohal::Error> {
//! use zohal::{Client, identity::ShahkarRequest};
//!
//! let client = Client::new(std::env::var("ZOHAL_TOKEN").unwrap_or_default())?;
//! let m = client
//!     .shahkar(&ShahkarRequest {
//!         mobile: "09120000000".into(),
//!         national_code: "0012345678".into(),
//!     })
//!     .await?;
//! println!("matched: {}", m.matched);
//! # Ok(())
//! # }
//! ```
//!
//! Every method maps to one operation of Zohal's OpenAPI document
//! (<https://zohal.io/documents>); its docs name the path and `operationId`.
//!
//! # Semantics
//!
//! - A method returns `Ok` only if the HTTP status is 2xx, `result` is 1 and
//!   `error_code` is null.  Zohal's documentation shows business failures
//!   such as `CARD_NOT_FOUND` returned with HTTP 200 and `result` 1, so a
//!   negative answer (`matched: false`) is never confused with a failed
//!   lookup; those come back as [`Error::Api`].
//! - Requests are authenticated with `Authorization: Bearer <token>`.
//!   Zohal also only answers requests from IP addresses you have registered
//!   with it.
//! - Nothing is retried.  Enquiries are billed per call and some, such as
//!   [`Client::voice_otp`], have effects outside Zohal; retrying is the
//!   caller's decision.
//! - Requests time out after [`DEFAULT_TIMEOUT`] unless configured
//!   otherwise, and response bodies are capped at [`MAX_RESPONSE_BYTES`].
//! - Inputs are not validated locally beyond what the types enforce; Zohal
//!   validates them and reports failures as [`Error::Api`].
//! - Dates are passed through as Zohal sends them, mostly Solar Hijri
//!   (e.g. `1405/01/15`).
//!
//! # TLS
//!
//! The default `rustls` feature uses reqwest's rustls stack.  Disable default
//! features and enable `native-tls` to use the platform library instead
//! (OpenSSL on FreeBSD).  One of the two is required; a build without
//! either fails to compile.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// build() only accepts https outside loopback, so without TLS the client
// could never reach Zohal.
#[cfg(not(any(feature = "rustls", feature = "native-tls")))]
compile_error!(
    "zohal needs a TLS backend: enable the `rustls` (default) or \
     `native-tls` feature"
);

macro_rules! string_enum {
    (
        $(#[$meta:meta])*
        $name:ident {
            $( $(#[$vmeta:meta])* $variant:ident = $value:literal, )*
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum $name {
            $( $(#[$vmeta])* $variant, )*
            /// A value not known to this version of the crate.
            Other(String),
        }

        impl $name {
            /// The wire value.
            pub fn as_str(&self) -> &str {
                match self {
                    $( $name::$variant => $value, )*
                    $name::Other(s) => s,
                }
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(
                &self,
                f: &mut ::std::fmt::Formatter<'_>,
            ) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                s.serialize_str(self.as_str())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(d: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let s = <String as ::serde::Deserialize>::deserialize(d)?;
                Ok(match s.as_str() {
                    $( $value => $name::$variant, )*
                    _ => $name::Other(s),
                })
            }
        }
    };
}

mod client;
mod error;
mod upload;

pub mod bank;
pub mod biometric;
pub mod general;
pub mod identity;

pub use client::{
    Client, ClientBuilder, DEFAULT_BASE_URL, DEFAULT_CONNECT_TIMEOUT,
    DEFAULT_TIMEOUT, MAX_RESPONSE_BYTES,
};
pub use error::{ApiError, Error, ResultCode};
pub use reqwest;
pub use upload::Upload;

use serde::{Deserialize, Serialize};

/// Outcome of a matching enquiry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Matched {
    /// Whether the inputs belong together.
    pub matched: bool,
}

/// Gender, as Zohal encodes it: `0` female, `1` male.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    /// `0`
    Female,
    /// `1`
    Male,
}

impl Serialize for Gender {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u8(match self {
            Gender::Female => 0,
            Gender::Male => 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biometric::SessionStatus;

    #[test]
    fn string_enum_keeps_unknown_values() {
        let known: SessionStatus = serde_json::from_str("\"pending\"").unwrap();
        assert_eq!(known, SessionStatus::Pending);
        let unknown: SessionStatus =
            serde_json::from_str("\"archived\"").unwrap();
        assert_eq!(unknown, SessionStatus::Other("archived".into()));
        for v in [known, unknown] {
            let wire = serde_json::to_string(&v).unwrap();
            assert_eq!(wire, format!("\"{}\"", v.as_str()));
            assert_eq!(v.to_string(), v.as_str());
        }
    }

    #[test]
    fn gender_wire_values() {
        assert_eq!(serde_json::to_string(&Gender::Female).unwrap(), "0");
        assert_eq!(serde_json::to_string(&Gender::Male).unwrap(), "1");
    }
}
