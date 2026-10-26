// SPDX-License-Identifier: BSD-2-Clause
// Copyright (c) 2026 Faraz Vahedi <kfv@FreeBSD.org>

//! Video identity verification (احراز هویت ویدیویی).
//!
//! The flow is: [`Client::upload_media`] a selfie video, start a session
//! with [`Client::start_liveness`] using the returned media ID, then wait
//! for Zohal's callback or poll [`Client::liveness_result`].
//!
//! These endpoints put the payload directly in `response_body` rather than
//! under `response_body.data`.

use reqwest::multipart::Form;
use serde::{Deserialize, Serialize};

use crate::client::Shape;
use crate::{Client, Error, Gender, Upload};

/// Kind of media for [`Client::upload_media`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MediaType {
    /// `selfie_video`
    SelfieVideo,
}

impl MediaType {
    /// The wire value.
    pub fn as_str(self) -> &'static str {
        match self {
            MediaType::SelfieVideo => "selfie_video",
        }
    }
}

/// Result of [`Client::upload_media`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Media {
    /// Media type, e.g. `selfie_video`.
    #[serde(rename = "type")]
    pub kind: String,
    /// Media ID (a UUID), passed on in [`LivenessMedia::selfie_video`].
    pub id: String,
}

/// Request for [`Client::start_liveness`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LivenessRequest {
    /// Previously uploaded media.
    pub media: LivenessMedia,
    /// Webhook through which Zohal announces the result when processing
    /// finishes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    /// Date of birth, Solar Hijri: `1377/06/30`, `1377-06-30`,
    /// `1377.06.30` or `13770630`.
    pub birth_date: String,
    /// National code, 10 digits.
    pub national_code: String,
    /// National card serial, 10 characters.
    pub national_card_serial: String,
    /// Gender.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<Gender>,
}

/// Media references for [`LivenessRequest`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LivenessMedia {
    /// [`Media::id`] of the selfie video.
    pub selfie_video: String,
}

/// Result of [`Client::start_liveness`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LivenessSession {
    /// Session ID for [`Client::liveness_result`].
    pub session_id: String,
    /// Processing state.
    pub status: SessionStatus,
}

/// Result of [`Client::liveness_result`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LivenessResult {
    /// Verdict, e.g. `matched` or `rejected`; empty while pending.
    pub result: String,
    /// Reason for the verdict.
    pub reason: LivenessReason,
    /// Session type, e.g. `Liveness`.
    #[serde(rename = "type")]
    pub kind: String,
    /// Processing state.
    pub status: SessionStatus,
    /// When processing finished.
    pub completed_at: Option<String>,
}

string_enum! {
    /// Processing state of a liveness session.
    SessionStatus {
        /// `pending`
        Pending = "pending",
        /// `waiting`
        Waiting = "waiting",
        /// `processing`
        Processing = "processing",
        /// `completed`
        Completed = "completed",
        /// `failed`
        Failed = "failed",
    }
}

string_enum! {
    /// Why a liveness session was accepted or rejected.
    LivenessReason {
        /// `ACCEPT`
        Accept = "ACCEPT",
        /// `REJECT_INPUT_DATA_INVALID`
        RejectInputDataInvalid = "REJECT_INPUT_DATA_INVALID",
        /// `REJECT_SERIAL_NATIONAL_CARD_WRONG`
        RejectSerialNationalCardWrong = "REJECT_SERIAL_NATIONAL_CARD_WRONG",
        /// `REJECT_NATIONAL_CARD_EXPIRED`
        RejectNationalCardExpired = "REJECT_NATIONAL_CARD_EXPIRED",
        /// `REJECT_INFO_NOT_MATCHED`
        RejectInfoNotMatched = "REJECT_INFO_NOT_MATCHED",
        /// `REJECT_FACE_NOT_MATCH_ID`
        RejectFaceNotMatchId = "REJECT_FACE_NOT_MATCH_ID",
        /// `REJECT_VIDEO_NOT_LIVE`
        RejectVideoNotLive = "REJECT_VIDEO_NOT_LIVE",
        /// `REJECT_VIDEO_BAD_LIGHT`
        RejectVideoBadLight = "REJECT_VIDEO_BAD_LIGHT",
        /// `REJECT_VIDEO_BAD_QUALITY`
        RejectVideoBadQuality = "REJECT_VIDEO_BAD_QUALITY",
        /// `REJECT_PERSON_TOO_FAR_AWAY`
        RejectPersonTooFarAway = "REJECT_PERSON_TOO_FAR_AWAY",
        /// `REJECT_MORE_THAN_ONE_PERSON`
        RejectMoreThanOnePerson = "REJECT_MORE_THAN_ONE_PERSON",
        /// `REJECT_NO_PERSON_DETECTED`
        RejectNoPersonDetected = "REJECT_NO_PERSON_DETECTED",
        /// `REJECT_INCOMPATIBLE_HEJAB`
        RejectIncompatibleHejab = "REJECT_INCOMPATIBLE_HEJAB",
        /// `REJECT_INCOMPATIBLE_ANGLE_CAMERA`
        RejectIncompatibleAngleCamera = "REJECT_INCOMPATIBLE_ANGLE_CAMERA",
        /// `REJECT_INCOMPATIBLE_BACKGROUND`
        RejectIncompatibleBackground = "REJECT_INCOMPATIBLE_BACKGROUND",
        /// `REJECT_INCOMPATIBLE_LOOK_CAMERA`
        RejectIncompatibleLookCamera = "REJECT_INCOMPATIBLE_LOOK_CAMERA",
        /// `REJECT_FAKE_VIDEO`
        RejectFakeVideo = "REJECT_FAKE_VIDEO",
        /// `REJECT_INVALID_SIGNATURE`
        RejectInvalidSignature = "REJECT_INVALID_SIGNATURE",
        /// `REJECT_USER_DID_NOT_SPEAK_REQUIRED_TEXT`
        RejectUserDidNotSpeakRequiredText = "REJECT_USER_DID_NOT_SPEAK_REQUIRED_TEXT",
        /// `SERVICE_UNAVAILABLE`
        ServiceUnavailable = "SERVICE_UNAVAILABLE",
        /// `UNDEFINED`
        Undefined = "UNDEFINED",
        /// `UNKNOWN`
        Unknown = "UNKNOWN",
    }
}

impl Client {
    /// Uploads media for a later liveness session.
    ///
    /// `POST /api/v0/services/biometric/media/` (`upload_media`).
    pub async fn upload_media(
        &self,
        kind: MediaType,
        file: Upload,
    ) -> Result<Media, Error> {
        let form = Form::new()
            .text("type", kind.as_str())
            .part("file", file.into_part());
        self.post_multipart(
            "/api/v0/services/biometric/media/",
            form,
            Shape::Bare,
        )
        .await
    }

    /// Starts a liveness session.
    ///
    /// `POST /api/v0/services/biometric/session/liveness/`
    /// (`start_liveness_check`).
    pub async fn start_liveness(
        &self,
        req: &LivenessRequest,
    ) -> Result<LivenessSession, Error> {
        self.post_json(
            "/api/v0/services/biometric/session/liveness/",
            req,
            Shape::Bare,
        )
        .await
    }

    /// Fetches the state of a liveness session.
    ///
    /// `GET /api/v0/services/biometric/session/{session_id}/result/`
    /// (`get_result`).
    pub async fn liveness_result(
        &self,
        session_id: &str,
    ) -> Result<LivenessResult, Error> {
        // Percent-encoding does not neutralise dot segments.
        if matches!(session_id, "" | "." | "..") {
            return Err(Error::InvalidArgument(
                "session ID is not a path segment",
            ));
        }
        self.get(
            [
                "api",
                "v0",
                "services",
                "biometric",
                "session",
                session_id,
                "result",
                "",
            ],
            Shape::Bare,
        )
        .await
    }
}
