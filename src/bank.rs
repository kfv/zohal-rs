// SPDX-License-Identifier: BSD-2-Clause
// Copyright (c) 2026 Faraz Vahedi <kfv@FreeBSD.org>

//! Banking enquiries (بانکی): cards, accounts, IBANs and cheques.

use serde::{Deserialize, Serialize};

use crate::client::Shape;
use crate::{Client, Error, Matched};

/// Request for the card enquiries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CardRequest {
    /// Card number, 16 digits.
    pub card_number: String,
}

/// Result of [`Client::card_to_iban`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CardIban {
    /// Holder's name.
    pub name: String,
    /// IBAN of the card's account.
    #[serde(rename = "IBAN")]
    pub iban: String,
    /// Bank name.
    pub bank_name: String,
}

/// Request for [`Client::iban`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IbanRequest {
    /// IBAN (Sheba), 26 characters including `IR`.
    pub iban: String,
    /// `separated` flag.  Zohal does not document it; the server defaults
    /// to `false` and it is not sent when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separated: Option<bool>,
}

/// Result of [`Client::iban`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IbanInfo {
    /// Account holder's name.
    pub name: String,
    /// Bank name.
    pub bank_name: String,
    /// Whether the account can receive transfers.
    pub is_transferable: bool,
}

/// Result of [`Client::card_holder`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CardHolder {
    /// Holder's name.
    pub name: String,
}

/// Request for [`Client::account_to_iban`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AccountRequest {
    /// Account number.
    pub bank_account: String,
    /// Three-digit bank code.  Zohal accepts the codes below.  The Persian
    /// names are Zohal's.  The English names are verbatim from the Central
    /// Bank of Iran's [list of banks and credit institutions][cbi] as of
    /// October 2026; "—" marks an institution that list does not include.
    ///
    /// | Code  | Zohal               | Central Bank of Iran            |
    /// |-------|---------------------|---------------------------------|
    /// | `010` | مرکزی               | Central Bank of Iran            |
    /// | `011` | صنعت و معدن         | Bank of Industry & Mine         |
    /// | `012` | ملت                 | Mellat Bank                     |
    /// | `013` | رفاه                | Refah Bank                      |
    /// | `014` | مسکن                | Bank Maskan                     |
    /// | `015` | سپه                 | Bank Sepah                      |
    /// | `016` | کشاورزی             | Bank Keshavarzi                 |
    /// | `017` | ملی                 | Bank Melli Iran                 |
    /// | `018` | تجارت               | Tejarat Bank                    |
    /// | `019` | صادرات              | Bank Saderat Iran               |
    /// | `020` | توسعه صادرات        | Export Development Bank of Iran |
    /// | `021` | پست بانک            | Post Bank of Iran               |
    /// | `022` | توسعه تعاون         | Cooperative Development Bank    |
    /// | `051` | موسسه اعتباری توسعه | —                               |
    /// | `052` | قوامین              | —                               |
    /// | `053` | کارآفرین            | Karafarin Bank                  |
    /// | `054` | پارسیان             | Parsian Bank                    |
    /// | `055` | اقتصاد نوین         | Eghtesad Novin Bank             |
    /// | `056` | سامان               | Saman Bank                      |
    /// | `057` | پاسارگاد            | Pasargad Bank                   |
    /// | `058` | سرمایه              | Sarmayeh Bank                   |
    /// | `059` | سینا                | Sina Bank                       |
    /// | `060` | قرض الحسنه مهر      | Gharzolhasaneh Mehr Iran Bank   |
    /// | `061` | شهر                 | Shahr Bank                      |
    /// | `062` | آینده               | Ayandeh Bank                    |
    /// | `063` | انصار               | —                               |
    /// | `064` | گردشگری             | Gardeshgari Bank                |
    /// | `065` | حکمت ایرانیان       | —                               |
    /// | `066` | دی                  | Day bank                        |
    /// | `069` | ایران زمین          | Iran Zamin Bank                 |
    /// | `070` | رسالت               | Gharzolhasaneh Resalat Bank     |
    /// | `073` | موسسه اعتباری کوثر  | —                               |
    /// | `075` | موسسه اعتباری ملل   | Melal Credit Institution        |
    /// | `078` | خاورمیانه           | Khavarmianeh Bank               |
    /// | `079` | مهر اقتصاد          | —                               |
    /// | `080` | موسسه اعتباری نور   | —                               |
    /// | `095` | ایران-ونزوئلا        | IRAN-VENEZUELA Bi-National Bank |
    ///
    /// [cbi]: https://www.cbi.ir/simplelist/1462.aspx
    pub bank_code: String,
}

/// Result of [`Client::account_to_iban`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccountIban {
    /// IBAN of the account.
    pub iban: String,
}

/// Request for [`Client::match_card_and_name`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CardNameRequest {
    /// Card number, 16 digits.
    pub card_number: String,
    /// Name to check.
    pub name: String,
}

/// Request for [`Client::match_iban_and_name`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IbanNameRequest {
    /// Name to check.
    pub name: String,
    /// IBAN (Sheba), 26 characters including `IR`.
    #[serde(rename = "IBAN")]
    pub iban: String,
}

/// Result of [`Client::card_to_account`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CardAccount {
    /// Holder's name.
    pub name: String,
    /// Account number.
    pub bank_account: String,
    /// Bank name.
    pub bank_name: String,
}

/// Request for [`Client::bounced_cheques`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BouncedChequesRequest {
    /// National code (10 digits) or company national ID (11 digits).
    pub national_code: String,
    /// `nationality_type`.  Zohal does not document its values.
    pub nationality_type: i64,
}

/// Result of [`Client::bounced_cheques`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BouncedCheques {
    /// Number of bounced cheques on record.
    pub count: u64,
}

/// Request for [`Client::sayad_cheque`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SayadRequest {
    /// Sayad ID, 16 digits.
    pub sayad_id: String,
}

/// Result of [`Client::sayad_cheque`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SayadCheque {
    /// Sayad ID.
    pub sayad_id: Option<String>,
    /// IBAN of the drawer's account.
    pub iban: Option<String>,
    /// Issue date.
    pub issue_date: Option<String>,
    /// Expiry date.
    pub expiration_date: Option<String>,
    /// Serial number.
    pub serial_no: Option<String>,
    /// Series number.
    pub series_no: Option<String>,
    /// Cheque type.
    pub check_type: Option<String>,
    /// Issuing branch code.
    pub branch_code: Option<String>,
    /// Drawer's name.
    pub name: Option<String>,
    /// Returned cheques on record.
    pub returned_cheques: Option<String>,
}

/// Request for [`Client::sayad_transfer_chain`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SayadChainRequest {
    /// Sayad ID, 16 digits.
    pub sayad_id: String,
    /// `cheque_type`.  Zohal does not document its values.
    pub cheque_type: String,
    /// National code (10 digits) or company national ID (11 digits).
    pub national_code: String,
}

/// Result of [`Client::sayad_transfer_chain`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SayadChain {
    /// Transfers, in the order Zohal returns them.
    pub chain: Vec<SayadChainItem>,
}

/// One link of a cheque's transfer chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SayadChainItem {
    /// Parties to this link.
    #[serde(default)]
    pub customers: Vec<SayadChainCustomer>,
    /// Role type code.
    pub role_type: Option<i64>,
}

/// A party in a cheque's transfer chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SayadChainCustomer {
    /// Name.
    pub name: Option<String>,
    /// National code.
    pub national_code: Option<String>,
    /// Customer type code.
    pub customer_type: Option<i64>,
}

/// Request for [`Client::cheque_submitted`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChequeSubmittedRequest {
    /// Sayad ID, 16 digits.
    pub sayad_id: String,
    /// National code (10 digits) or company national ID (11 digits).
    pub national_code: String,
}

/// Result of [`Client::cheque_submitted`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChequeSubmitted {
    /// Whether the cheque is registered in Sayad.
    pub status: bool,
}

impl Client {
    /// Converts a card number to its account's IBAN.
    ///
    /// `POST /api/v0/services/inquiry/card_to_iban` (`convert-card-to-iban`).
    pub async fn card_to_iban(
        &self,
        req: &CardRequest,
    ) -> Result<CardIban, Error> {
        self.post_json(
            "/api/v0/services/inquiry/card_to_iban",
            req,
            Shape::Data,
        )
        .await
    }

    /// Looks up an IBAN's holder and bank.
    ///
    /// `POST /api/v0/services/inquiry/iban` (`iban-inquiry`).
    pub async fn iban(&self, req: &IbanRequest) -> Result<IbanInfo, Error> {
        self.post_json("/api/v0/services/inquiry/iban", req, Shape::Data)
            .await
    }

    /// Looks up a card holder's name.
    ///
    /// `POST /api/v0/services/inquiry/card_inquiry` (`card-inquiry`).
    pub async fn card_holder(
        &self,
        req: &CardRequest,
    ) -> Result<CardHolder, Error> {
        self.post_json(
            "/api/v0/services/inquiry/card_inquiry",
            req,
            Shape::Data,
        )
        .await
    }

    /// Converts an account number to an IBAN.
    ///
    /// `POST /api/v0/services/inquiry/account_to_iban`
    /// (`convert-account-number-to-iban`).
    pub async fn account_to_iban(
        &self,
        req: &AccountRequest,
    ) -> Result<AccountIban, Error> {
        self.post_json(
            "/api/v0/services/inquiry/account_to_iban",
            req,
            Shape::Data,
        )
        .await
    }

    /// Checks that a card belongs to a name.
    ///
    /// `POST /api/v0/services/inquiry/check_card_with_name`
    /// (`match-name-and-payment-card`).
    pub async fn match_card_and_name(
        &self,
        req: &CardNameRequest,
    ) -> Result<Matched, Error> {
        self.post_json(
            "/api/v0/services/inquiry/check_card_with_name",
            req,
            Shape::Data,
        )
        .await
    }

    /// Checks that an IBAN belongs to a name.
    ///
    /// `POST /api/v0/services/inquiry/check_iban_with_name`
    /// (`match-iban-and-name`).
    pub async fn match_iban_and_name(
        &self,
        req: &IbanNameRequest,
    ) -> Result<Matched, Error> {
        self.post_json(
            "/api/v0/services/inquiry/check_iban_with_name",
            req,
            Shape::Data,
        )
        .await
    }

    /// Converts a card number to its account number.
    ///
    /// `POST /api/v0/services/inquiry/card_to_account`
    /// (`convert-card-to-account-number`).
    pub async fn card_to_account(
        &self,
        req: &CardRequest,
    ) -> Result<CardAccount, Error> {
        self.post_json(
            "/api/v0/services/inquiry/card_to_account",
            req,
            Shape::Data,
        )
        .await
    }

    /// Counts a person's or company's bounced cheques.
    ///
    /// `POST /api/v0/services/inquiry/bounced_cheque`
    /// (`bounced-cheque-inquiry`).
    pub async fn bounced_cheques(
        &self,
        req: &BouncedChequesRequest,
    ) -> Result<BouncedCheques, Error> {
        self.post_json(
            "/api/v0/services/inquiry/bounced_cheque",
            req,
            Shape::Data,
        )
        .await
    }

    /// Looks up a Sayad cheque.
    ///
    /// `POST /api/v0/services/inquiry/check_sayad_inquiry`
    /// (`check-sayad-inquiry`).
    pub async fn sayad_cheque(
        &self,
        req: &SayadRequest,
    ) -> Result<SayadCheque, Error> {
        self.post_json(
            "/api/v0/services/inquiry/check_sayad_inquiry",
            req,
            Shape::Data,
        )
        .await
    }

    /// Lists a Sayad cheque's transfer chain.
    ///
    /// `POST /api/v0/services/inquiry/check_sayad_inquiry/chain`
    /// (`sayad-transfers-chain-inquiry`).
    pub async fn sayad_transfer_chain(
        &self,
        req: &SayadChainRequest,
    ) -> Result<SayadChain, Error> {
        self.post_json(
            "/api/v0/services/inquiry/check_sayad_inquiry/chain",
            req,
            Shape::Data,
        )
        .await
    }

    /// Checks whether a Sayad cheque has been registered.
    ///
    /// `POST /api/v0/services/inquiry/check_sayad_inquiry/submitted`
    /// (`cheque-submitted-inquiry`).
    pub async fn cheque_submitted(
        &self,
        req: &ChequeSubmittedRequest,
    ) -> Result<ChequeSubmitted, Error> {
        self.post_json(
            "/api/v0/services/inquiry/check_sayad_inquiry/submitted",
            req,
            Shape::Data,
        )
        .await
    }
}
