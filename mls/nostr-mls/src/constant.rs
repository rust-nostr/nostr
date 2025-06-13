//! Nostr MLS constants

use openmls::extensions::ExtensionType;
use openmls_traits::types::Ciphersuite;

/// Nostr Group Data extension type
pub const NOSTR_GROUP_DATA_EXTENSION_TYPE: u16 = 0xF2EE; // Be FREE

/// Default ciphersuite for Nostr Groups.
/// This is also the only required ciphersuite for Nostr Groups.
pub const DEFAULT_CIPHERSUITE: Ciphersuite =
    Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

/// Required extensions for Nostr Groups.
pub const REQUIRED_EXTENSIONS: [ExtensionType; 4] = [
    ExtensionType::RequiredCapabilities,
    ExtensionType::LastResort,
    ExtensionType::RatchetTree,
    ExtensionType::Unknown(NOSTR_GROUP_DATA_EXTENSION_TYPE),
];

// /// GREASE values for MLS.
// TODO: Remove this once we've added GREASE support.
// const GREASE: [u16; 15] = [
//     0x0A0A, 0x1A1A, 0x2A2A, 0x3A3A, 0x4A4A, 0x5A5A, 0x6A6A, 0x7A7A, 0x8A8A, 0x9A9A, 0xAAAA,
//     0xBABA, 0xCACA, 0xDADA, 0xEAEA,
// ];
