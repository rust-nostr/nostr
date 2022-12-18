// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

#![doc = include_str!("../README.md")]

#[macro_use]
extern crate serde;

pub use bitcoin::hashes;
pub use bitcoin::hashes::sha256::Hash as Sha256Hash;
pub use bitcoin::secp256k1;
pub use url;

pub mod contact;
pub mod entity;
pub mod event;
pub mod key;
pub mod message;
pub mod metadata;
pub mod util;

pub use self::contact::Contact;
pub use self::entity::Entity;
pub use self::event::{Event, EventBuilder, Kind, KindBase, Tag};
pub use self::key::Keys;
pub use self::message::{ClientMessage, RelayMessage, SubscriptionFilter};
pub use self::metadata::Metadata;

pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::secp256k1::SecretKey;

    use super::Result;
    use crate::{Event, EventBuilder, Keys, RelayMessage};

    #[test]
    fn parse_message() -> Result<()> {
        // Got this fresh off the wire
        pub const SAMPLE_EVENT: &'static str = r#"["EVENT", "random_string", {"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"}]"#;

        // Hand parsed version as a sanity check
        let id = "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5";
        let pubkey = "379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe";
        let created_at = 1612809991;
        let kind = 1;
        let tags = vec![];
        let content = "test";
        let sig = "273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502";

        let event = Event::new_dummy(id, pubkey, created_at, kind, tags, content, sig);

        let parsed_event = RelayMessage::from_json(SAMPLE_EVENT);

        assert_eq!(
            parsed_event.expect("Failed to parse event"),
            RelayMessage::new_event(event?, "random_string".to_string())
        );

        Ok(())
    }

    #[test]
    fn round_trip() -> Result<()> {
        let keys = Keys::new(SecretKey::from_str(
            "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
        )?);

        let event = EventBuilder::new_text_note("hello", &vec![]).to_event(&keys)?;

        let serialized = event.as_json().unwrap();
        let deserialized = Event::from_json(serialized)?;

        assert_eq!(event, deserialized);

        Ok(())
    }

    #[test]
    #[cfg(feature = "nip04")]
    fn test_encrypted_direct_msg() -> Result<()> {
        let sender_keys = Keys::new(SecretKey::from_str(
            "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
        )?);
        let receiver_keys = Keys::new(SecretKey::from_str(
            "7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
        )?);

        let content = "Mercury, the Winged Messenger";
        let event = EventBuilder::new_encrypted_direct_msg(&sender_keys, &receiver_keys, content)?
            .to_event(&sender_keys)?;

        Ok(event.verify()?)
    }
}
