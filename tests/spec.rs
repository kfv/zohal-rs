// SPDX-License-Identifier: BSD-2-Clause
// Copyright (c) 2026 Faraz Vahedi <kfv@FreeBSD.org>

//! Checks the client against the vendored OpenAPI document: every operation
//! is implemented, requests match the documented method, path and body, and
//! every documented example and schema-generated response is classified and
//! decoded as the success rule says.

use std::collections::BTreeSet;

use serde_json::{Value, json};

use zohal::bank::*;
use zohal::biometric::*;
use zohal::general::*;
use zohal::identity::*;
use zohal::{Client, Error, Gender, Upload};

mod common;

use common::{Captured, serve_once};

const SPEC: &str = include_str!("../spec/openapi.json");
const TOKEN: &str = "test-token";
const SESSION: &str = "sess-1";

fn s(v: &str) -> String {
    v.to_owned()
}

fn upload() -> Upload {
    Upload::from_bytes("x.bin", b"\x00\x01\x02".to_vec())
        .content_type("application/octet-stream")
        .unwrap()
}

/// Calls the method for `op` with plausible arguments, discarding the
/// payload.
async fn call(c: &Client, op: &str) -> Result<(), Error> {
    let card = || CardRequest {
        card_number: s("6037990000000000"),
    };
    let company = || CompanyRequest {
        national_id: s("10123456789"),
    };
    let bill = || BillRequest {
        bill_id: s("123456789"),
    };
    let mobile = || MobileRequest {
        mobile: s("09120000000"),
    };
    let vehicle = || VehicleRequest {
        national_code: s("0012345678"),
        mobile: s("09120000000"),
        region_code: s("11"),
        plate_number: s("12b34567"),
    };
    match op {
        "shahkar-inquiry" => c
            .shahkar(&ShahkarRequest {
                mobile: s("09120000000"),
                national_code: s("0012345678"),
            })
            .await
            .map(drop),
        "ocr-for-id-card" => c
            .national_card_ocr(NationalCardOcrRequest {
                front: Some(CardImage::File(upload())),
                back: Some(CardImage::Url(s("https://example.test/b.jpg"))),
            })
            .await
            .map(drop),
        "company-inquiry" => c.company(&company()).await.map(drop),
        "company-inquiry-board-members" => {
            c.company_board_members(&company()).await.map(drop)
        }
        "company-inquiry-board-members-history" => {
            c.company_board_members_history(&company()).await.map(drop)
        }
        "match-payment-card-number-and-id-number" => c
            .match_card_and_national_code(&CardNationalCodeRequest {
                national_code: s("0012345678"),
                birth_date: s("1377/06/30"),
                card_number: s("6037990000000000"),
            })
            .await
            .map(drop),
        "match-iban-and-id-number" => c
            .match_iban_and_national_code(&IbanNationalCodeRequest {
                national_code: s("0012345678"),
                birth_date: s("1377/06/30"),
                iban: s("IR000000000000000000000000"),
            })
            .await
            .map(drop),
        "id-inquiry" => c
            .national_identity(&NationalIdentityRequest {
                birth_date: s("1377/06/30"),
                national_code: s("0012345678"),
                gender: Some(false),
            })
            .await
            .map(drop),
        "upload_media" => c
            .upload_media(MediaType::SelfieVideo, upload())
            .await
            .map(drop),
        "start_liveness_check" => c
            .start_liveness(&LivenessRequest {
                media: LivenessMedia {
                    selfie_video: s("3fa85f64-5717-4562-b3fc-2c963f66afa6"),
                },
                callback_url: Some(s("https://example.test/cb")),
                birth_date: s("1377/06/30"),
                national_code: s("0012345678"),
                national_card_serial: s("1A23456789"),
                gender: Some(Gender::Female),
            })
            .await
            .map(drop),
        "get_result" => c.liveness_result(SESSION).await.map(drop),
        "convert-card-to-iban" => c.card_to_iban(&card()).await.map(drop),
        "iban-inquiry" => c
            .iban(&IbanRequest {
                iban: s("IR000000000000000000000000"),
                separated: Some(false),
            })
            .await
            .map(drop),
        "card-inquiry" => c.card_holder(&card()).await.map(drop),
        "convert-account-number-to-iban" => c
            .account_to_iban(&AccountRequest {
                bank_account: s("1234567890"),
                bank_code: s("012"),
            })
            .await
            .map(drop),
        "match-name-and-payment-card" => c
            .match_card_and_name(&CardNameRequest {
                card_number: s("6037990000000000"),
                name: s("x"),
            })
            .await
            .map(drop),
        "match-iban-and-name" => c
            .match_iban_and_name(&IbanNameRequest {
                name: s("x"),
                iban: s("IR000000000000000000000000"),
            })
            .await
            .map(drop),
        "convert-card-to-account-number" => {
            c.card_to_account(&card()).await.map(drop)
        }
        "bounced-cheque-inquiry" => c
            .bounced_cheques(&BouncedChequesRequest {
                national_code: s("0012345678"),
                nationality_type: 1,
            })
            .await
            .map(drop),
        "check-sayad-inquiry" => c
            .sayad_cheque(&SayadRequest {
                sayad_id: s("1234567890123456"),
            })
            .await
            .map(drop),
        "sayad-transfers-chain-inquiry" => c
            .sayad_transfer_chain(&SayadChainRequest {
                sayad_id: s("1234567890123456"),
                cheque_type: s("1"),
                national_code: s("0012345678"),
            })
            .await
            .map(drop),
        "cheque-submitted-inquiry" => c
            .cheque_submitted(&ChequeSubmittedRequest {
                sayad_id: s("1234567890123456"),
                national_code: s("0012345678"),
            })
            .await
            .map(drop),
        "postal-code-inquiry" => c
            .postal_code(&PostalCodeRequest {
                postal_code: s("1234567890"),
            })
            .await
            .map(drop),
        "vehicle-inquiry-total-violations" => {
            c.vehicle_violations_total(&vehicle()).await.map(drop)
        }
        "vehicle-inquiry-violations-details" => {
            c.vehicle_violations_details(&vehicle()).await.map(drop)
        }
        "enamad-inquiry" => c
            .enamad(&EnamadRequest {
                website: s("example.ir"),
            })
            .await
            .map(drop),
        "convert-persian-name-to-english" => c
            .persian_to_english(&PersianToEnglishRequest {
                persian_text: s("علی"),
            })
            .await
            .map(drop),
        "electricity-bill-inquiry" => {
            c.electricity_bill(&bill()).await.map(drop)
        }
        "fixed-line-bill-inquiry" => c
            .fixed_line_bill(&FixedLineRequest {
                phone_number: s("02112345678"),
            })
            .await
            .map(drop),
        "gas-bill-inquiry" => c.gas_bill(&bill()).await.map(drop),
        "irancell-bill-inquiry" => c.irancell_bill(&mobile()).await.map(drop),
        "mci-bill-inquiry" => c.mci_bill(&mobile()).await.map(drop),
        "rightel-bill-inquiry" => c.rightel_bill(&mobile()).await.map(drop),
        "water-bill-inquiry" => c.water_bill(&bill()).await.map(drop),
        "voice-otp" => c
            .voice_otp(&VoiceOtpRequest {
                mobile: s("09120000000"),
                code: s("1234"),
            })
            .await
            .map(drop),
        other => panic!("no client method for operation {other}"),
    }
}

const IMPLEMENTED: &[&str] = &[
    "shahkar-inquiry",
    "ocr-for-id-card",
    "company-inquiry",
    "company-inquiry-board-members",
    "company-inquiry-board-members-history",
    "match-payment-card-number-and-id-number",
    "match-iban-and-id-number",
    "id-inquiry",
    "upload_media",
    "start_liveness_check",
    "get_result",
    "convert-card-to-iban",
    "iban-inquiry",
    "card-inquiry",
    "convert-account-number-to-iban",
    "match-name-and-payment-card",
    "match-iban-and-name",
    "convert-card-to-account-number",
    "bounced-cheque-inquiry",
    "check-sayad-inquiry",
    "sayad-transfers-chain-inquiry",
    "cheque-submitted-inquiry",
    "postal-code-inquiry",
    "vehicle-inquiry-total-violations",
    "vehicle-inquiry-violations-details",
    "enamad-inquiry",
    "convert-persian-name-to-english",
    "electricity-bill-inquiry",
    "fixed-line-bill-inquiry",
    "gas-bill-inquiry",
    "irancell-bill-inquiry",
    "mci-bill-inquiry",
    "rightel-bill-inquiry",
    "water-bill-inquiry",
    "voice-otp",
];

fn resolve<'a>(spec: &'a Value, schema: &'a Value) -> &'a Value {
    match schema.get("$ref").and_then(Value::as_str) {
        Some(r) => {
            let name = r.strip_prefix("#/components/schemas/").unwrap();
            resolve(spec, &spec["components"]["schemas"][name])
        }
        None => schema,
    }
}

fn is_null_schema(spec: &Value, s: &Value) -> bool {
    resolve(spec, s).get("type").and_then(Value::as_str) == Some("null")
}

/// Generates an instance of `schema`.  `minimal` leaves out optional
/// properties and uses null wherever null is allowed; otherwise every
/// property is present and non-null.
fn generate(spec: &Value, schema: &Value, minimal: bool) -> Value {
    let schema = resolve(spec, schema);
    for key in ["anyOf", "oneOf"] {
        if let Some(alts) = schema.get(key).and_then(Value::as_array) {
            if minimal && alts.iter().any(|a| is_null_schema(spec, a)) {
                return Value::Null;
            }
            let alt = alts.iter().find(|a| !is_null_schema(spec, a)).unwrap();
            return generate(spec, alt, minimal);
        }
    }
    if minimal && schema.get("nullable") == Some(&Value::Bool(true)) {
        return Value::Null;
    }
    if let Some(e) = schema.get("enum").and_then(Value::as_array) {
        return e[0].clone();
    }
    match schema.get("type").and_then(Value::as_str) {
        Some("object") => {
            let required: BTreeSet<&str> = schema
                .get("required")
                .and_then(Value::as_array)
                .map(|r| r.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let mut obj = serde_json::Map::new();
            if let Some(props) =
                schema.get("properties").and_then(Value::as_object)
            {
                for (k, v) in props {
                    if !minimal || required.contains(k.as_str()) {
                        obj.insert(k.clone(), generate(spec, v, minimal));
                    }
                }
            }
            Value::Object(obj)
        }
        Some("array") => {
            if minimal {
                json!([])
            } else {
                json!([generate(spec, &schema["items"], minimal)])
            }
        }
        Some("string") => match schema.get("format").and_then(Value::as_str) {
            Some("date-time") => json!("2024-01-01T12:00:00"),
            Some("uuid") => json!("3fa85f64-5717-4562-b3fc-2c963f66afa6"),
            _ => json!("x"),
        },
        Some("integer") => json!(1),
        Some("number") => json!(1.5),
        Some("boolean") => json!(true),
        other => panic!("cannot generate {other:?}: {schema}"),
    }
}

/// Checks a request value against its schema.
fn check_value(
    spec: &Value,
    schema: &Value,
    v: &Value,
    at: &str,
    errs: &mut Vec<String>,
) {
    let schema = resolve(spec, schema);
    if let Some(e) = schema.get("enum").and_then(Value::as_array) {
        if !e.contains(v) {
            errs.push(format!("{at}: {v} not in {e:?}"));
        }
        return;
    }
    let ok = match schema.get("type").and_then(Value::as_str) {
        Some("object") => {
            let Some(obj) = v.as_object() else {
                errs.push(format!("{at}: expected object, got {v}"));
                return;
            };
            check_fields(
                spec,
                schema,
                obj.keys().map(String::as_str),
                at,
                errs,
            );
            if let Some(props) =
                schema.get("properties").and_then(Value::as_object)
            {
                for (k, sub) in obj {
                    if let Some(ps) = props.get(k) {
                        check_value(spec, ps, sub, &format!("{at}.{k}"), errs);
                    }
                }
            }
            true
        }
        Some("string") => v.is_string(),
        Some("integer") => v.is_i64() || v.is_u64(),
        Some("boolean") => v.is_boolean(),
        Some("array") => v.is_array(),
        _ => true,
    };
    if !ok {
        errs.push(format!("{at}: {v} does not match {schema}"));
    }
}

/// Checks a set of field names against an object schema.
fn check_fields<'a>(
    spec: &Value,
    schema: &Value,
    names: impl Iterator<Item = &'a str>,
    at: &str,
    errs: &mut Vec<String>,
) {
    let schema = resolve(spec, schema);
    let names: BTreeSet<&str> = names.collect();
    let props: BTreeSet<&str> = schema["properties"]
        .as_object()
        .map(|p| p.keys().map(String::as_str).collect())
        .unwrap_or_default();
    for n in names.difference(&props) {
        errs.push(format!("{at}: undocumented field {n}"));
    }
    if let Some(req) = schema.get("required").and_then(Value::as_array) {
        for r in req.iter().filter_map(Value::as_str) {
            if !names.contains(r) {
                errs.push(format!("{at}: missing required field {r}"));
            }
        }
    }
}

fn multipart_names(body: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(body);
    text.split("form-data; name=\"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .map(str::to_owned)
        .collect()
}

fn check_request(
    spec: &Value,
    method: &str,
    path: &str,
    op: &Value,
    got: &Captured,
) -> Vec<String> {
    let mut errs = Vec::new();
    let want_target = path.replace("{session_id}", SESSION);
    if got.method != method.to_uppercase() || got.target != want_target {
        errs.push(format!(
            "sent {} {}, spec says {} {}",
            got.method,
            got.target,
            method.to_uppercase(),
            want_target
        ));
    }
    if got.header("authorization") != Some(&format!("Bearer {TOKEN}")) {
        errs.push(format!(
            "authorization header {:?}",
            got.header("authorization")
        ));
    }
    if got.header("accept") != Some("application/json") {
        errs.push(format!("accept header {:?}", got.header("accept")));
    }
    let Some(content) = op["requestBody"]["content"].as_object() else {
        if !got.body.is_empty() {
            errs.push("body sent where none is documented".into());
        }
        return errs;
    };
    let ct = got.header("content-type").unwrap_or("");
    let media = ct.split(';').next().unwrap_or("").trim();
    let Some(schema) = content.get(media).map(|c| &c["schema"]) else {
        errs.push(format!("content type {media} not documented"));
        return errs;
    };
    if media == "application/json" {
        let v: Value = serde_json::from_slice(&got.body).unwrap();
        check_value(spec, schema, &v, "body", &mut errs);
    } else {
        let names = multipart_names(&got.body);
        check_fields(
            spec,
            schema,
            names.iter().map(String::as_str),
            "form",
            &mut errs,
        );
    }
    errs
}

/// What the client must do with a response, per the success rule.
fn expect_ok(status: u16, body: &Value) -> bool {
    let rb = &body["response_body"];
    (200..300).contains(&status)
        && body["result"] == json!(1)
        && rb.get("error_code").is_none_or(Value::is_null)
}

#[tokio::test]
async fn conforms_to_spec() {
    let spec: Value = serde_json::from_str(SPEC).unwrap();
    let mut failures = Vec::new();
    let mut seen = BTreeSet::new();
    let mut exchanges = 0usize;

    for (path, item) in spec["paths"].as_object().unwrap() {
        for (method, op) in item.as_object().unwrap() {
            let id = op["operationId"].as_str().unwrap();
            seen.insert(id.to_owned());
            if !IMPLEMENTED.contains(&id) {
                failures.push(format!("{id}: not implemented"));
                continue;
            }

            let mut cases: Vec<(String, u16, Value)> = Vec::new();
            for (code, resp) in op["responses"].as_object().unwrap() {
                let status: u16 = code.parse().unwrap();
                let media = &resp["content"]["application/json"];
                if let Some(examples) = media["examples"].as_object() {
                    for (name, ex) in examples {
                        cases.push((
                            format!("example {code} {name}"),
                            status,
                            ex["value"].clone(),
                        ));
                    }
                }
                if status == 200 {
                    for minimal in [false, true] {
                        let mut v = generate(&spec, &media["schema"], minimal);
                        v["result"] = json!(1);
                        if let Some(rb) = v["response_body"].as_object_mut()
                            && rb.contains_key("data")
                        {
                            rb.insert("error_code".into(), Value::Null);
                        }
                        let label = if minimal { "minimal" } else { "full" };
                        cases.push((format!("generated {label}"), 200, v));
                    }
                }
            }

            for (label, status, body) in cases {
                exchanges += 1;
                let (base, server) =
                    serve_once(status, serde_json::to_vec(&body).unwrap())
                        .await;
                let client =
                    Client::builder(TOKEN).base_url(base).build().unwrap();
                let got = call(&client, id).await;
                let captured = server.await.unwrap();
                for e in check_request(&spec, method, path, op, &captured) {
                    failures.push(format!("{id} [{label}]: {e}"));
                }
                let want = expect_ok(status, &body);
                match (&got, want) {
                    (Ok(()), true) => {}
                    (Err(Error::Api(e)), false) => {
                        let code = body["response_body"]["error_code"].as_str();
                        if e.status != status || e.code.as_deref() != code {
                            failures.push(format!(
                                "{id} [{label}]: wrong ApiError {e:?}"
                            ));
                        }
                    }
                    _ => failures.push(format!(
                        "{id} [{label}]: expected {}, got {got:?}",
                        if want { "Ok" } else { "Api error" }
                    )),
                }
            }
        }
    }

    let implemented: BTreeSet<String> =
        IMPLEMENTED.iter().map(|s| s.to_string()).collect();
    for gone in implemented.difference(&seen) {
        failures.push(format!("{gone}: implemented but no longer in the spec"));
    }
    assert!(exchanges > 300, "only {exchanges} exchanges");
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The bank code table in [`AccountRequest`]'s docs matches the spec,
/// row for row.
#[test]
fn bank_code_docs_match_spec() {
    let spec: Value = serde_json::from_str(SPEC).unwrap();
    let schema = &spec["components"]["schemas"]["BankCodeEnum"];
    let doc = include_str!("../src/bank.rs");
    // Rows are `| code | Zohal name | English name |`, padded however the
    // table is aligned.  Code and Zohal name must match the spec, and every
    // row needs an English cell.
    let mut got = BTreeSet::new();
    for line in doc.lines().map(str::trim_start) {
        let Some(row) = line.strip_prefix("///") else {
            continue;
        };
        let cells: Vec<&str> = row
            .trim()
            .strip_prefix('|')
            .and_then(|r| r.strip_suffix('|'))
            .map(|r| r.split('|').map(str::trim).collect())
            .unwrap_or_default();
        let Some(code) = cells.first().and_then(|c| c.strip_prefix('`')) else {
            continue; // not a table row, or the header or separator
        };
        assert_eq!(cells.len(), 3, "row {line:?}");
        assert!(!cells[2].is_empty(), "row {line:?}");
        let code = code.strip_suffix('`').unwrap_or(code);
        got.insert((code.to_owned(), cells[1].to_owned()));
    }
    let mut want = BTreeSet::new();
    for line in schema["description"].as_str().unwrap().lines() {
        let (code, name) = line
            .strip_prefix("* `")
            .and_then(|l| l.split_once("` - "))
            .unwrap_or_else(|| panic!("unexpected line {line:?}"));
        want.insert((code.to_owned(), name.to_owned()));
    }
    let codes: BTreeSet<&str> = schema["enum"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect();
    assert_eq!(want.len(), codes.len(), "description and enum disagree");
    assert_eq!(got, want);
}

/// Every enum value the spec defines maps to a named variant, not `Other`,
/// and goes back out unchanged; so does every value the client sends.
#[test]
fn enums_cover_spec() {
    let spec: Value = serde_json::from_str(SPEC).unwrap();
    let values = |name: &str| -> Vec<Value> {
        spec["components"]["schemas"][name]["enum"]
            .as_array()
            .unwrap_or_else(|| panic!("no enum {name}"))
            .clone()
    };
    for v in values("ReasonChoices") {
        let r: LivenessReason = serde_json::from_value(v.clone()).unwrap();
        assert!(!matches!(r, LivenessReason::Other(_)), "{v}");
        assert_eq!(serde_json::to_value(&r).unwrap(), v);
    }
    for v in values("StatusChoices") {
        let s: SessionStatus = serde_json::from_value(v.clone()).unwrap();
        assert!(!matches!(s, SessionStatus::Other(_)), "{v}");
        assert_eq!(serde_json::to_value(&s).unwrap(), v);
    }
    let genders: Vec<Value> = [Gender::Female, Gender::Male]
        .iter()
        .map(|g| serde_json::to_value(g).unwrap())
        .collect();
    assert_eq!(genders, values("GenderEnum"));
    assert_eq!(
        vec![json!(MediaType::SelfieVideo.as_str())],
        values("BiometricMediaRequestTypeEnum")
    );
}
