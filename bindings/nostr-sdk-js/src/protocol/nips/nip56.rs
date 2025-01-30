// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::prelude::*;
use wasm_bindgen::prelude::*;

/// Report
///
/// <https://github.com/nostr-protocol/nips/blob/master/56.md>
#[wasm_bindgen(js_name = Report)]
pub enum JsReport {
    /// Depictions of nudity, porn, etc
    Nudity,
    /// Virus, trojan horse, worm, robot, spyware, adware, back door, ransomware, rootkit, kidnapper, etc.
    Malware,
    /// Profanity, hateful speech, etc.
    Profanity,
    /// Something which may be illegal in some jurisdiction
    Illegal,
    /// Spam
    Spam,
    /// Someone pretending to be someone else
    Impersonation,
    ///  Reports that don't fit in the above categories
    Other,
}

impl From<JsReport> for Report {
    fn from(value: JsReport) -> Self {
        match value {
            JsReport::Nudity => Self::Nudity,
            JsReport::Malware => Self::Malware,
            JsReport::Profanity => Self::Profanity,
            JsReport::Illegal => Self::Illegal,
            JsReport::Spam => Self::Spam,
            JsReport::Impersonation => Self::Impersonation,
            JsReport::Other => Self::Other,
        }
    }
}
