use nostr::prelude::*;

pub fn run() -> Result<()> {
    // Sender keys
    let alice_keys =
        Keys::parse("5c0c523f52a5b6fad39ed2403092df8cebc36318b39383bca6c00808626fab3a")?;

    // Receiver Keys
    let bob_keys = Keys::parse("nsec1j4c6269y9w0q2er2xjw8sv2ehyrtfxq3jwgdlxj6qfn8z4gjsq5qfvfk99")?;

    // Compose rumor
    let rumor: UnsignedEvent =
        EventBuilder::text_note("Test", []).to_unsigned_event(alice_keys.public_key());

    // Build gift wrap with sender keys
    let gw: Event = EventBuilder::gift_wrap(&alice_keys, &bob_keys.public_key(), rumor, None)?;
    println!("Gift Wrap: {}", gw.as_json());

    // Extract rumor from gift wrap with receiver keys
    let UnwrappedGift { sender, rumor } = nip59::extract_rumor(&bob_keys, &gw)?;
    println!("Sender: {sender}");
    println!("Rumor: {}", rumor.as_json());

    Ok(())
}
