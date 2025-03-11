// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP42: Authentication of clients to relays
//!
//! <https://github.com/nostr-protocol/nips/blob/master/42.md>

use crate::{Event, Kind, RelayUrl, TagKind, TagStandard};

/// Check if the [`Event`] is a valid authentication.
///
/// This function checks for:
/// - event kind, that must be [`Kind::Authentication`];
/// - `relay` tag, that must match `relay_url` arg;
/// - `challenge` tag, that must match `challenge` arg.
///
/// If all the above checks pass, returns `true`.
pub fn is_valid_auth_event(event: &Event, relay_url: &RelayUrl, challenge: &str) -> bool {
    // Check event kind
    if event.kind != Kind::Authentication {
        return false;
    }

    // Check if it has "relay" tag
    match event.tags.find_standardized(TagKind::Relay) {
        Some(TagStandard::Relay(url)) => {
            if url != relay_url {
                return false;
            }
        }
        Some(..) | None => return false,
    }

    // Check if it has the challenge
    match event.tags.find_standardized(TagKind::Challenge) {
        Some(TagStandard::Challenge(c)) => {
            if c != challenge {
                return false;
            }
        }
        Some(..) | None => return false,
    }

    // Valid
    true
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::{EventBuilder, Keys};

    #[test]
    fn test_valid_auth_event() {
        let keys = Keys::generate();
        let relay_url = RelayUrl::parse("wss://relay.damus.io").unwrap();
        let challenge = "1234567890";

        let event = EventBuilder::auth(challenge, relay_url.clone())
            .sign_with_keys(&keys)
            .unwrap();

        assert!(is_valid_auth_event(&event, &relay_url, challenge));
    }

    #[test]
    fn test_invalid_auth_event() {
        let keys = Keys::generate();
        let relay_url = RelayUrl::parse("wss://relay.damus.io").unwrap();
        let challenge = "1234567890";

        // Wrong challenge
        let event = EventBuilder::auth("abcd", relay_url.clone())
            .sign_with_keys(&keys)
            .unwrap();
        assert!(!is_valid_auth_event(&event, &relay_url, challenge));

        // Wrong relay url
        let event = EventBuilder::auth(challenge, RelayUrl::parse("wss://example.com").unwrap())
            .sign_with_keys(&keys)
            .unwrap();
        assert!(!is_valid_auth_event(&event, &relay_url, challenge));

        // Wrong kind
        let event = EventBuilder::text_note("abcd")
            .sign_with_keys(&keys)
            .unwrap();
        assert!(!is_valid_auth_event(&event, &relay_url, challenge));
    }
}
