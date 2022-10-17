// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

#[macro_use]
extern crate serde;

pub mod event;
mod key;
mod message;
pub mod util;

pub use crate::event::{Event, Kind, KindBase, Contact};
pub use crate::key::Keys;
pub use crate::message::{ClientMessage, RelayMessage, SubscriptionFilter};

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::str::FromStr;

    use secp256k1::SecretKey;

    use crate::{Event, Keys, RelayMessage};

    type TestResult = Result<(), Box<dyn Error>>;

    #[test]
    fn parse_message() -> TestResult {
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
    fn round_trip() -> TestResult {
        let keys = Keys::new(SecretKey::from_str(
            "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
        )?);

        let event = Event::new_textnote("hello", &keys, &vec![])?;

        let serialized = event.as_json();
        let deserialized = Event::new_from_json(serialized)?;

        assert_eq!(event, deserialized);

        Ok(())
    }

    #[test]
    fn test_encrypted_direct_msg() -> TestResult {
        let sender_keys = Keys::new(SecretKey::from_str(
            "6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
        )?);
        let receiver_keys = Keys::new(SecretKey::from_str(
            "7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e",
        )?);

        let content = "Mercury, the Winged Messenger";
        let event = Event::new_encrypted_direct_msg(&sender_keys, &receiver_keys, content);

        assert_eq!(event?.verify(), Ok(()));

        Ok(())
    }
}
