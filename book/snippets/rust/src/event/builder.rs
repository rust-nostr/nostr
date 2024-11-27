use nostr_sdk::prelude::*;

pub fn event() -> Result<()> {
    let keys = Keys::generate();

    // Compose custom event
    let custom_event = EventBuilder::new(Kind::Custom(1111), "").sign_with_keys(&keys)?;

    // Compose text note
    let textnote_event = EventBuilder::text_note("Hello").sign_with_keys(&keys)?;

    // Compose reply to above text note
    let reply_event = EventBuilder::text_note("Reply to hello")
        .tag(Tag::event(textnote_event.id))
        .sign_with_keys(&keys)?;

    // Compose POW event
    let pow_event =
        EventBuilder::text_note("Another reply with POW")
            .tag(Tag::event(textnote_event.id))
            .pow(20)
            .sign_with_keys(&keys)?;

    Ok(())
}
