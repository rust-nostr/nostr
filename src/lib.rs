mod event;
mod message;

pub use crate::event::Event;
pub use crate::message::Message;

#[cfg(test)]
mod tests {
    use secp256k1::rand::rngs::OsRng;
    use secp256k1::{schnorrsig, Secp256k1};

    use crate::{Event, Message};

    #[test]
    fn parse_message() {
        // Got this fresh off the wire
        pub const SAMPLE_EVENT: &'static str = r#"[{"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"},"n"]"#;

        // Hand parsed version as a sanity check
        let id = "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5";
        let pubkey = "379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe";
        let created_at = 1612809991;
        let kind = 1;
        let tags = vec![];
        let content = "test";
        let sig = "273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502";

        let event = Event::new_dummy(id, pubkey, created_at, kind, tags, content, sig);

        let parsed_event = Message::handle(SAMPLE_EVENT);

        assert_eq!(
            parsed_event.expect("Failed to parse event"),
            Message::Event(event)
        );
    }

    #[test]
    fn round_trip() {
        let secp = Secp256k1::new();
        let mut rng = OsRng::new().expect("OsRng");
        let keypair = schnorrsig::KeyPair::new(&secp, &mut rng);

        let event = Event::new_textnote("hello", &keypair);

        let serialized = event.as_json();
        let deserialized = Event::new_from_json(serialized).unwrap();

        assert_eq!(event, deserialized);
    }
}
