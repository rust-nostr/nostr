// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr::nips::nip56;
use uniffi::Enum;

/// Report
///
/// <https://github.com/nostr-protocol/nips/blob/master/56.md>
#[derive(Enum)]
pub enum Report {
    /// Depictions of nudity, porn, etc
    Nudity,
    /// Virus, trojan horse, worm, robot, spyware, adware, back door, ransomware, rootkit, kidnapper, etc.
    Malware,
    /// Profanity, hateful speech, etc.
    Profanity,
    /// Something which may be illegal in some jurisdiction
    ///
    /// Remember: there is what is right and there is the law.
    Illegal,
    /// Spam
    Spam,
    /// Someone pretending to be someone else
    Impersonation,
    /// Reports that don't fit in the above categories
    Other,
}

impl From<Report> for nip56::Report {
    fn from(value: Report) -> Self {
        match value {
            Report::Nudity => Self::Nudity,
            Report::Malware => Self::Malware,
            Report::Profanity => Self::Profanity,
            Report::Illegal => Self::Illegal,
            Report::Spam => Self::Spam,
            Report::Impersonation => Self::Impersonation,
            Report::Other => Self::Other,
        }
    }
}

impl From<nip56::Report> for Report {
    fn from(value: nip56::Report) -> Self {
        match value {
            nip56::Report::Nudity => Self::Nudity,
            nip56::Report::Malware => Self::Malware,
            nip56::Report::Profanity => Self::Profanity,
            nip56::Report::Illegal => Self::Illegal,
            nip56::Report::Spam => Self::Spam,
            nip56::Report::Impersonation => Self::Impersonation,
            nip56::Report::Other => Self::Other,
        }
    }
}
