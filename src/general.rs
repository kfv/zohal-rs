// SPDX-License-Identifier: BSD-2-Clause
// Copyright (c) 2026 Faraz Vahedi <kfv@FreeBSD.org>

//! General enquiries (خدماتی): addresses, vehicles, Enamad, bills and
//! voice OTP.  Amounts are passed through as Zohal returns them; it does
//! not document their currency.

use serde::{Deserialize, Serialize};

use crate::client::Shape;
use crate::{Client, Error};

/// Request for [`Client::postal_code`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostalCodeRequest {
    /// Postal code, 10 digits.
    pub postal_code: String,
}

/// Result of [`Client::postal_code`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PostalCode {
    /// Address registered for the postal code.
    pub address: Address,
}

/// A postal address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Address {
    /// Province.
    pub province: Option<String>,
    /// City.
    pub city: Option<String>,
    /// Town.
    pub town: Option<String>,
    /// District.
    pub district: Option<String>,
    /// Street.
    pub street: Option<String>,
    /// Second street.
    pub street2: Option<String>,
    /// Building number.
    pub number: Option<String>,
    /// Floor.
    pub floor: Option<String>,
    /// Unit on the floor.
    pub side_floor: Option<String>,
    /// Building name.
    pub building_name: Option<String>,
    /// Free-form description.
    pub description: Option<String>,
}

/// Request for the vehicle violation enquiries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VehicleRequest {
    /// Owner's national code, 10 digits.
    pub national_code: String,
    /// Owner's mobile number, 11 digits.
    pub mobile: String,
    /// Plate region code, 2 digits.
    pub region_code: String,
    /// Plate number.
    pub plate_number: String,
}

/// Result of [`Client::vehicle_violations_total`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VehicleViolationsTotal {
    /// Plate.
    pub plate: String,
    /// Payment status.
    pub price_status: String,
    /// Number of report pages.
    pub page_count: i64,
    /// Document ID.
    pub paper_id: String,
    /// Payment ID.
    pub payment_id: String,
    /// Fine amount.
    pub warning_price: String,
    /// Amount payable.
    pub inquire_price: String,
    /// Enforcement tracking number.
    pub ejr_inquire_no: String,
}

/// Result of [`Client::vehicle_violations_details`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VehicleViolationsDetails {
    /// Individual violations.
    pub warnings: Vec<VehicleWarning>,
}

/// A single traffic violation.  Field names follow Zohal's, typos
/// included.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VehicleWarning {
    /// Serial number.
    pub serial_no: String,
    /// Date of the violation.
    pub violation_occure_date: String,
    /// Time of the violation.
    pub violation_occure_time: String,
    /// How the notice was delivered.
    pub violation_delivery_type: ViolationDeliveryType,
    /// Kind of violation.
    pub violation_type: ViolationType,
    /// Amount payable.
    pub final_price: String,
    /// Document ID.
    pub paper_id: String,
    /// Payment ID.
    pub payment_id: String,
    /// Whether an image is available.
    pub has_image: bool,
    /// Fine ID.
    pub warning_id: String,
    /// Whether the violation can be reviewed.
    pub investigation_ability: bool,
    /// Where it happened.
    pub violatoin_address: Option<String>,
}

/// Delivery of a violation notice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ViolationDeliveryType {
    /// Delivery method.
    pub violation_delivery_type: Option<String>,
}

/// Kind of traffic violation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ViolationType {
    /// Violation type code.
    pub violation_type_id: String,
    /// Description.
    pub violation_type: String,
}

/// Request for [`Client::enamad`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnamadRequest {
    /// Website domain.
    pub website: String,
}

/// Result of [`Client::enamad`].  A site without Enamad is a successful
/// answer with only `message` set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Enamad {
    /// Status text, e.g. whether the site holds Enamad.
    pub message: String,
    /// Whether the certificate has expired.
    pub expired: Option<bool>,
    /// Registered activity.
    pub srv_text: Option<String>,
    /// Province.
    pub state_name: Option<String>,
    /// City.
    pub city_name: Option<String>,
    /// Approval date.
    pub approve_date: Option<String>,
    /// Expiry date.
    pub expiry_date: Option<String>,
}

/// Request for [`Client::persian_to_english`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PersianToEnglishRequest {
    /// Name in Persian script.
    pub persian_text: String,
}

/// Result of [`Client::persian_to_english`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Finglish {
    /// Name in Latin script.
    pub finglish_text: String,
}

/// Request for the electricity, gas and water bill enquiries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BillRequest {
    /// Bill ID.
    pub bill_id: String,
}

/// Result of [`Client::electricity_bill`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ElectricityBill {
    /// Address.
    pub address: String,
    /// Amount.
    pub amount: i64,
    /// Subscriber's name.
    pub full_name: String,
    /// Bill ID.
    pub bill_id: String,
    /// Current date.
    pub current_date: String,
    /// Previous date.
    pub previous_date: String,
    /// Payment ID.
    pub payment_id: Option<String>,
    /// Payment date.
    pub payment_date: String,
    /// Account type.
    pub account_type: String,
}

/// Result of [`Client::gas_bill`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GasBill {
    /// Address.
    pub address: String,
    /// Amount.
    pub amount: i64,
    /// Subscriber's name.
    pub full_name: String,
    /// Bill ID.
    pub bill_id: String,
    /// Current reading date.
    pub current_reading_date: String,
    /// Previous reading date.
    pub previous_reading_date: String,
    /// Payment date.
    pub payment_date: String,
    /// Consumption type.
    pub consumption_type: String,
    /// Payment ID.
    pub payment_id: Option<String>,
    /// Previous meter reading.
    pub previous_counter_value: Option<String>,
    /// Current meter reading.
    pub current_counter_value: Option<String>,
    /// Subscription.
    pub subscription: Option<String>,
    /// Taxes.
    pub taxes: Option<String>,
    /// Insurance.
    pub insurance: Option<String>,
}

/// Result of [`Client::water_bill`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WaterBill {
    /// Address.
    pub address: String,
    /// Amount.
    pub amount: i64,
    /// Bill ID.
    pub bill_id: String,
    /// Current date.
    pub current_date: String,
    /// Previous date.
    pub previous_date: String,
    /// Payment ID.
    pub payment_id: String,
    /// Payment date.
    pub payment_date: String,
    /// Subscriber's name.
    pub full_name: String,
    /// Account type.
    pub account_type: String,
}

/// Request for [`Client::fixed_line_bill`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FixedLineRequest {
    /// Landline number.
    pub phone_number: String,
}

/// Request for the mobile bill enquiries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileRequest {
    /// Mobile number, 11 digits.
    pub mobile: String,
}

/// Result of the fixed-line and mobile bill enquiries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PhoneBill {
    /// End-of-term bill.
    pub final_term: Option<PhoneBillTerm>,
    /// Mid-term bill.
    pub mid_term: Option<PhoneBillTerm>,
}

/// One term of a phone bill.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PhoneBillTerm {
    /// Amount.
    pub amount: i64,
    /// Bill ID.
    pub bill_id: String,
    /// Payment ID.
    pub payment_id: String,
}

/// Request for [`Client::voice_otp`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VoiceOtpRequest {
    /// Mobile number to call, 11 digits.
    pub mobile: String,
    /// Code to read out.
    pub code: String,
}

/// Result of [`Client::voice_otp`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VoiceOtp {
    /// Status text.
    pub message: String,
}

impl Client {
    /// Looks up the address for a postal code.
    ///
    /// `POST /api/v0/services/inquiry/postal_code_inquiry`
    /// (`postal-code-inquiry`).
    pub async fn postal_code(
        &self,
        req: &PostalCodeRequest,
    ) -> Result<PostalCode, Error> {
        self.post_json(
            "/api/v0/services/inquiry/postal_code_inquiry",
            req,
            Shape::Data,
        )
        .await
    }

    /// Totals a vehicle's traffic fines.
    ///
    /// `POST /api/v0/services/inquiry/vehicle_inquiry/total_violations`
    /// (`vehicle-inquiry-total-violations`).
    pub async fn vehicle_violations_total(
        &self,
        req: &VehicleRequest,
    ) -> Result<VehicleViolationsTotal, Error> {
        self.post_json(
            "/api/v0/services/inquiry/vehicle_inquiry/total_violations",
            req,
            Shape::Data,
        )
        .await
    }

    /// Lists a vehicle's traffic violations.
    ///
    /// `POST /api/v0/services/inquiry/vehicle_inquiry/violations_details`
    /// (`vehicle-inquiry-violations-details`).
    pub async fn vehicle_violations_details(
        &self,
        req: &VehicleRequest,
    ) -> Result<VehicleViolationsDetails, Error> {
        self.post_json(
            "/api/v0/services/inquiry/vehicle_inquiry/violations_details",
            req,
            Shape::Data,
        )
        .await
    }

    /// Looks up a website's Enamad trust seal.
    ///
    /// `POST /api/v0/services/inquiry/enamad_inquiry` (`enamad-inquiry`).
    pub async fn enamad(&self, req: &EnamadRequest) -> Result<Enamad, Error> {
        self.post_json(
            "/api/v0/services/inquiry/enamad_inquiry",
            req,
            Shape::Data,
        )
        .await
    }

    /// Transliterates a Persian name into Latin script.
    ///
    /// `POST /api/v0/services/inquiry/persian_to_finglish`
    /// (`convert-persian-name-to-english`).
    pub async fn persian_to_english(
        &self,
        req: &PersianToEnglishRequest,
    ) -> Result<Finglish, Error> {
        self.post_json(
            "/api/v0/services/inquiry/persian_to_finglish",
            req,
            Shape::Data,
        )
        .await
    }

    /// Fetches an electricity bill.
    ///
    /// `POST /api/v0/services/inquiry/bill/electricity`
    /// (`electricity-bill-inquiry`).
    pub async fn electricity_bill(
        &self,
        req: &BillRequest,
    ) -> Result<ElectricityBill, Error> {
        self.post_json(
            "/api/v0/services/inquiry/bill/electricity",
            req,
            Shape::Data,
        )
        .await
    }

    /// Fetches a landline bill.
    ///
    /// `POST /api/v0/services/inquiry/bill/fixed_line`
    /// (`fixed-line-bill-inquiry`).
    pub async fn fixed_line_bill(
        &self,
        req: &FixedLineRequest,
    ) -> Result<PhoneBill, Error> {
        self.post_json(
            "/api/v0/services/inquiry/bill/fixed_line",
            req,
            Shape::Data,
        )
        .await
    }

    /// Fetches a gas bill.
    ///
    /// `POST /api/v0/services/inquiry/bill/gas` (`gas-bill-inquiry`).
    pub async fn gas_bill(&self, req: &BillRequest) -> Result<GasBill, Error> {
        self.post_json("/api/v0/services/inquiry/bill/gas", req, Shape::Data)
            .await
    }

    /// Fetches an Irancell bill.
    ///
    /// `POST /api/v0/services/inquiry/bill/irancell`
    /// (`irancell-bill-inquiry`).
    pub async fn irancell_bill(
        &self,
        req: &MobileRequest,
    ) -> Result<PhoneBill, Error> {
        self.post_json(
            "/api/v0/services/inquiry/bill/irancell",
            req,
            Shape::Data,
        )
        .await
    }

    /// Fetches a Hamrah-e Aval (MCI) bill.
    ///
    /// `POST /api/v0/services/inquiry/bill/mci` (`mci-bill-inquiry`).
    pub async fn mci_bill(
        &self,
        req: &MobileRequest,
    ) -> Result<PhoneBill, Error> {
        self.post_json("/api/v0/services/inquiry/bill/mci", req, Shape::Data)
            .await
    }

    /// Fetches a Rightel bill.
    ///
    /// `POST /api/v0/services/inquiry/bill/rightel` (`rightel-bill-inquiry`).
    pub async fn rightel_bill(
        &self,
        req: &MobileRequest,
    ) -> Result<PhoneBill, Error> {
        self.post_json(
            "/api/v0/services/inquiry/bill/rightel",
            req,
            Shape::Data,
        )
        .await
    }

    /// Fetches a water bill.
    ///
    /// `POST /api/v0/services/inquiry/bill/water` (`water-bill-inquiry`).
    pub async fn water_bill(
        &self,
        req: &BillRequest,
    ) -> Result<WaterBill, Error> {
        self.post_json("/api/v0/services/inquiry/bill/water", req, Shape::Data)
            .await
    }

    /// Places a call that reads a one-time code to a mobile number.  Each
    /// call is billed and rings the phone; do not retry blindly.
    ///
    /// `POST /api/v0/services/inquiry/voice_otp` (`voice-otp`).
    pub async fn voice_otp(
        &self,
        req: &VoiceOtpRequest,
    ) -> Result<VoiceOtp, Error> {
        self.post_json("/api/v0/services/inquiry/voice_otp", req, Shape::Data)
            .await
    }
}
