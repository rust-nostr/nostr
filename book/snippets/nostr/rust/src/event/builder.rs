use nostr::prelude::*;

pub fn event() -> Result<()> {
    let keys = Keys::generate();

    // Compose custom event
    let custom_event = EventBuilder::new(Kind::Custom(1111), "", []).to_event(&keys)?;

    // Compose text note
    let textnote_event = EventBuilder::text_note("Hello", []).to_event(&keys)?;

    // Compose reply to above text note
    let reply_event = EventBuilder::text_note("Reply to hello", [Tag::event(textnote_event.id)])
        .to_event(&keys)?;

    // Compose POW event
    let pow_event =
        EventBuilder::text_note("Another reply with POW", [Tag::event(textnote_event.id)])
            .to_pow_event(&keys, 20)?;

    Ok(())
}
