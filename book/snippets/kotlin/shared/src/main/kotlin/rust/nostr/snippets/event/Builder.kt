package rust.nostr.snippets

import rust.nostr.sdk.*

// ANCHOR: builder
fun builder() {
    val keys = Keys.generate();

    // Compose custom event
    val customEvent = EventBuilder(kind = Kind(1111u), content = "").signWithKeys(keys);

    // Compose text note
    val textNoteEvent = EventBuilder.textNote("Hello").signWithKeys(keys);

    // Compose reply to above text note
    val replyEvent = EventBuilder.textNote("Reply to hello")
        .tags(listOf(Tag.event(textNoteEvent.id())))
        .signWithKeys(keys);

    // Compose POW event
    val powEvent =
    EventBuilder.textNote("Another reply with POW")
        .tags(listOf(Tag.event(textNoteEvent.id())))
        .pow(20u)
        .signWithKeys(keys);
    println(powEvent.asJson())
}
// ANCHOR_END: builder
