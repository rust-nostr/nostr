mod error;
mod event;

pub use crate::event::Event;

#[cfg(test)]
mod tests {
    use crate::event::{handle_incoming_message, Event, Kind, NostrMessage};

    #[test]
    fn parse_message() {
        // let hash_of_hello = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
pub const SAMPLE_EVENT: &'static str = r#"[{"id":"70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5","pubkey":"379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe","created_at":1612809991,"kind":1,"tags":[],"content":"test","sig":"273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502"},"n"]"#;



        let id = "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5";
        let pubkey = "379e863e8357163b5bce5d2688dc4f1dcc2d505222fb8d74db600f30535dfdfe";
        let created_at = 1612809991; 
        let kind = 1;
        let tags = vec![];
        let content = "test";
        let sig = "273a9cd5d11455590f4359500bccb7a89428262b96b3ea87a756b770964472f8c3e87f5d5e64d8d2e859a71462a3f477b554565c4f2f326cb01dd7620db71502";

        let event = Event::new_dummy(
            id,
            pubkey,
            created_at,
            kind,
            tags,
            content,
            sig,
        );

        let message = tungstenite::Message::from(SAMPLE_EVENT);

        let parsed_event = handle_incoming_message(message);

        assert_eq!(parsed_event.expect("Failed to parse event"), NostrMessage::Event(event));
    }

    #[test]
    fn round_trip() {
        let event = Event::new("hello");

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);

    }
}
