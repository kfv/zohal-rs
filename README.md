# zohal

Unofficial Rust client for the [Zohal](https://zohal.io) enquiry API:
identity verification (Shahkar, national identity, card and IBAN ownership,
national card OCR, video liveness, company records), banking (card, account
and IBAN lookups and conversions, Sayad and bounced cheques) and general
enquiries (postal codes, vehicle fines, Enamad, name transliteration, bills,
voice OTP).

Not affiliated with or endorsed by Zohal.

```toml
[dependencies]
zohal = "0.1"
```

```rust
use zohal::{Client, identity::ShahkarRequest};

async fn owns_number(token: String) -> Result<bool, zohal::Error> {
    let client = Client::new(token)?;
    let m = client
        .shahkar(&ShahkarRequest {
            mobile: "09120000000".into(),
            national_code: "0012345678".into(),
        })
        .await?;
    Ok(m.matched)
}
```

## Behaviour

- A call succeeds only if the HTTP status is 2xx, `result` is 1 and
  `error_code` is null.  Zohal's documentation shows some failures, such as
  `CARD_NOT_FOUND`, returned with HTTP 200 and `result` 1; those are errors
  here, so a failed lookup is never mistaken for `matched: false`.
- Requests carry `Authorization: Bearer <token>`.  Zohal only answers
  requests from IP addresses you have registered with it.
- Nothing is retried: enquiries are billed per call and some have effects
  outside Zohal.
- Requests time out after 30 seconds by default; responses are capped at
  8 MiB.

## TLS

One TLS backend is required; a build without either fails to compile.  The
default `rustls` feature uses reqwest's rustls stack.  For the platform
library instead (OpenSSL on FreeBSD):

```toml
zohal = { version = "0.1", default-features = false, features = ["native-tls"] }
```

## Supported Rust versions

The latest stable release and the two before it ("N-2").  `rust-version`
is currently 1.97, and is raised as new releases come out.

## API coverage

All 35 operations in Zohal's OpenAPI document as published at
<https://zohal.io/documents> on 26 October 2026.  The document is kept in
`spec/` as JSON and YAML, and `tests/spec.rs` checks the client against
it.

## Licence

Licensed under BSD-2-Clause (see `LICENSE`), except for the files under
`spec/`, which are owned by Zohal and included here for reference only.
