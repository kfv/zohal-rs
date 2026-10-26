// SPDX-License-Identifier: BSD-2-Clause
// Copyright (c) 2026 Faraz Vahedi <kfv@FreeBSD.org>

//! Identity verification (احراز هویت): people and companies.

use reqwest::multipart::Form;
use serde::{Deserialize, Serialize};

use crate::client::Shape;
use crate::{Client, Error, Matched, Upload};

/// Request for [`Client::shahkar`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShahkarRequest {
    /// Mobile number, 11 digits.
    pub mobile: String,
    /// National code, 10 digits.
    pub national_code: String,
}

/// One side of a national card for [`Client::national_card_ocr`].
#[derive(Debug)]
pub enum CardImage {
    /// An image uploaded with the request.
    File(Upload),
    /// An image Zohal fetches itself.
    Url(String),
}

/// Request for [`Client::national_card_ocr`].
#[derive(Debug, Default)]
pub struct NationalCardOcrRequest {
    /// Front of the card.
    pub front: Option<CardImage>,
    /// Back of the card.
    pub back: Option<CardImage>,
}

/// Result of [`Client::national_card_ocr`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NationalCardOcr {
    /// Data read from the front, if it was sent.
    pub front: Option<NationalCardFront>,
    /// Data read from the back, if it was sent.
    pub back: Option<NationalCardBack>,
}

/// Front of a national card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NationalCardFront {
    /// Surname.
    pub last_name: String,
    /// Date of birth.
    pub birth_date: String,
    /// Given name.
    pub first_name: String,
    /// Father's name.
    pub father_name: String,
    /// National code.
    pub national_code: String,
    /// Card expiry date.
    pub expiration_date: Option<String>,
    /// City.
    pub city: Option<String>,
    /// Province.
    pub province: Option<String>,
    /// Holder's photo, Base64.
    pub face_photo: Option<String>,
}

/// Back of a national card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NationalCardBack {
    /// Card serial.
    pub serial_card: Option<String>,
}

/// Request for the company enquiries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CompanyRequest {
    /// Company national ID, 11 digits.
    pub national_id: String,
}

/// Result of [`Client::company`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Company {
    /// Date activity ended.
    pub activity_end_date: Option<String>,
    /// Registered address.
    pub address: Option<String>,
    /// Legal form.
    pub company_type: Option<String>,
    /// Creation date.
    pub created_at: Option<String>,
    /// Email address.
    pub email_address: Option<String>,
    /// Fax number.
    pub fax_number: Option<String>,
    /// Phone number.
    pub phone_number: Option<String>,
    /// Issuance date.
    pub issuance_date: Option<String>,
    /// Registered name.
    pub name: Option<String>,
    /// National ID.
    pub national_id: Option<String>,
    /// Postal code.
    pub postal_code: Option<String>,
    /// Registration date.
    pub register_date: Option<String>,
    /// Registration number.
    pub register_number: Option<String>,
}

/// Result of [`Client::company_board_members`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BoardMembers {
    /// Company name.
    pub company_title: String,
    /// Current board.
    pub board_members: Vec<BoardMember>,
}

/// A current board member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BoardMember {
    /// Position held.
    pub position: String,
    /// Full name.
    pub name: String,
    /// Start of the term.
    pub start_date: String,
    /// Length of the term.
    pub duration: Option<String>,
}

/// Result of [`Client::company_board_members_history`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BoardMembersHistory {
    /// Company name.
    pub company_title: String,
    /// Past and present board members.
    pub board_members: Vec<BoardMemberHistory>,
}

/// A board member's term.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BoardMemberHistory {
    /// Position held.
    pub position: String,
    /// Full name.
    pub name: String,
    /// Start of the term.
    pub start_date: String,
    /// Length of the term.
    pub duration: Option<String>,
    /// End of the term.
    pub end_date: Option<String>,
}

/// Request for [`Client::match_card_and_national_code`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CardNationalCodeRequest {
    /// National code, 10 digits.
    pub national_code: String,
    /// Date of birth, Solar Hijri: `1377/06/30`, `1377-06-30`,
    /// `1377.06.30` or `13770630`.
    pub birth_date: String,
    /// Card number, 16 digits.
    pub card_number: String,
}

/// Request for [`Client::match_iban_and_national_code`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IbanNationalCodeRequest {
    /// National code, 10 digits.
    pub national_code: String,
    /// Date of birth, Solar Hijri: `1377/06/30`, `1377-06-30`,
    /// `1377.06.30` or `13770630`.
    pub birth_date: String,
    /// IBAN (Sheba), 26 characters including `IR`.
    #[serde(rename = "IBAN")]
    pub iban: String,
}

/// Request for [`Client::national_identity`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NationalIdentityRequest {
    /// Date of birth, Solar Hijri: `1377/06/30`, `1377-06-30`,
    /// `1377.06.30` or `13770630`.
    pub birth_date: String,
    /// National code, 10 digits.
    pub national_code: String,
    /// `gender` flag.  Zohal does not document it; the server defaults to
    /// `false` and it is not sent when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<bool>,
}

/// Result of [`Client::national_identity`].  When `matched` is false the
/// other fields are `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NationalIdentity {
    /// Whether the national code and date of birth belong together.
    pub matched: bool,
    /// Given name.
    pub first_name: Option<String>,
    /// National code.
    pub national_code: Option<String>,
    /// Surname.
    pub last_name: Option<String>,
    /// Father's name.
    pub father_name: Option<String>,
    /// Whether the person is alive.
    pub alive: Option<bool>,
    /// Whether the person is recorded as dead.
    pub is_dead: Option<bool>,
}

impl Client {
    /// Checks that a mobile number is registered to a national code
    /// (Shahkar).
    ///
    /// `POST /api/v0/services/inquiry/shahkar` (`shahkar-inquiry`).
    pub async fn shahkar(
        &self,
        req: &ShahkarRequest,
    ) -> Result<Matched, Error> {
        self.post_json("/api/v0/services/inquiry/shahkar", req, Shape::Data)
            .await
    }

    /// Reads a national card from images of either or both sides.
    ///
    /// `POST /api/v0/services/inquiry/national_card_ocr` (`ocr-for-id-card`).
    pub async fn national_card_ocr(
        &self,
        req: NationalCardOcrRequest,
    ) -> Result<NationalCardOcr, Error> {
        let mut form = Form::new();
        for (name, side) in [
            ("national_card_front", req.front),
            ("national_card_back", req.back),
        ] {
            form = match side {
                Some(CardImage::File(u)) => form.part(name, u.into_part()),
                Some(CardImage::Url(url)) => form.text(name, url),
                None => form,
            };
        }
        self.post_multipart(
            "/api/v0/services/inquiry/national_card_ocr",
            form,
            Shape::Data,
        )
        .await
    }

    /// Looks up a company's registration record.
    ///
    /// `POST /api/v0/services/inquiry/company_inquiry` (`company-inquiry`).
    pub async fn company(
        &self,
        req: &CompanyRequest,
    ) -> Result<Company, Error> {
        self.post_json(
            "/api/v0/services/inquiry/company_inquiry",
            req,
            Shape::Data,
        )
        .await
    }

    /// Lists a company's current board.
    ///
    /// `POST /api/v0/services/inquiry/company_inquiry/board_members`
    /// (`company-inquiry-board-members`).
    pub async fn company_board_members(
        &self,
        req: &CompanyRequest,
    ) -> Result<BoardMembers, Error> {
        self.post_json(
            "/api/v0/services/inquiry/company_inquiry/board_members",
            req,
            Shape::Data,
        )
        .await
    }

    /// Lists a company's board over time.
    ///
    /// `POST /api/v0/services/inquiry/company_inquiry/board_members/history`
    /// (`company-inquiry-board-members-history`).
    pub async fn company_board_members_history(
        &self,
        req: &CompanyRequest,
    ) -> Result<BoardMembersHistory, Error> {
        self.post_json(
            "/api/v0/services/inquiry/company_inquiry/board_members/history",
            req,
            Shape::Data,
        )
        .await
    }

    /// Checks that a payment card belongs to a national code.
    ///
    /// `POST /api/v0/services/inquiry/check_card_with_national_code`
    /// (`match-payment-card-number-and-id-number`).
    pub async fn match_card_and_national_code(
        &self,
        req: &CardNationalCodeRequest,
    ) -> Result<Matched, Error> {
        self.post_json(
            "/api/v0/services/inquiry/check_card_with_national_code",
            req,
            Shape::Data,
        )
        .await
    }

    /// Checks that an IBAN belongs to a national code.
    ///
    /// `POST /api/v0/services/inquiry/check_iban_with_national_code`
    /// (`match-iban-and-id-number`).
    pub async fn match_iban_and_national_code(
        &self,
        req: &IbanNationalCodeRequest,
    ) -> Result<Matched, Error> {
        self.post_json(
            "/api/v0/services/inquiry/check_iban_with_national_code",
            req,
            Shape::Data,
        )
        .await
    }

    /// Checks a national code against a date of birth and returns the
    /// civil registry record.
    ///
    /// `POST /api/v0/services/inquiry/national_identity_inquiry`
    /// (`id-inquiry`).
    pub async fn national_identity(
        &self,
        req: &NationalIdentityRequest,
    ) -> Result<NationalIdentity, Error> {
        self.post_json(
            "/api/v0/services/inquiry/national_identity_inquiry",
            req,
            Shape::Data,
        )
        .await
    }
}
