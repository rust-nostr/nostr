use serde::{Deserialize, Serialize};

/// A descriptor for the blob
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlobDescriptor {
    /// The URL at which the blob/file can be accessed
    pub url: String,
    /// The SHA256 hash of the contents in the blob
    pub sha256: String,
    /// The size of the blob/file, in bytes
    pub size: u32,
    #[serde(rename = "type")]
    /// Mime type of the blob/file
    pub mime_type: Option<String>,
    /// The date at which the blob was uploaded, as a UNIX timestamp (in seconds)
    pub uploaded: nostr::Timestamp,
}
