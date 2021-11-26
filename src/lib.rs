mod event;
mod message;
mod user;

pub mod util;
pub use crate::event::Event;
pub use crate::event::Kind;
pub use crate::message::ClientMessage;
pub use crate::message::RelayMessage;
pub use crate::message::SubscriptionFilter;
pub use crate::user::gen_keys;

#[cfg(test)]
mod tests {
    use secp256k1::rand::rngs::OsRng;
    use secp256k1::{schnorrsig, Secp256k1, SecretKey};
    use std::str::FromStr;

    use crate::{Event, RelayMessage};

    #[test]
    fn parse_message() {
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
            RelayMessage::new_event(event, "random_string".to_string())
        );
    }

    #[test]
    fn round_trip() {
        let secp = Secp256k1::new();
        let mut rng = OsRng::new().expect("OsRng");
        let keypair = schnorrsig::KeyPair::new(&secp, &mut rng);

        let event = Event::new_textnote("hello", &keypair).unwrap();

        let serialized = event.as_json();
        let deserialized = Event::new_from_json(serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_encrypted_direct_msg() {
        let secp = Secp256k1::new();
        let sender_sk =
            SecretKey::from_str("6b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();
        let receiver_sk =
            SecretKey::from_str("7b911fd37cdf5c81d4c0adb1ab7fa822ed253ab0ad9aa18d77257c88b29b718e")
                .unwrap();
        let receiver_key_pair = schnorrsig::KeyPair::from_secret_key(&secp, receiver_sk);
        let receiver_pk = schnorrsig::PublicKey::from_keypair(&secp, &receiver_key_pair);

        let content = "Mercury, the Winged Messenger";
        let event = Event::new_encrypted_direct_msg(sender_sk, &receiver_pk, content);

        assert_eq!(event.verify(), Ok(()));
    }
}
