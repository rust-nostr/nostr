import { Keys, EventBuilder, Tag, Timestamp, Kind } from "@rust-nostr/nostr-sdk"

export function eventBuilder() {
    let keys = Keys.generate();

    // Compose custom event
    let kind = new Kind(1111);
    let customEvent = new EventBuilder(kind, "").signWithKeys(keys);

    // Compose text note
    let textnoteEvent = EventBuilder.textNote("Hello").signWithKeys(keys);

    // Compose reply to above text note
    let replyEvent =
        EventBuilder.textNote("Reply to hello")
            .tags([Tag.event(textnoteEvent.id)])
            .signWithKeys(keys);

    // Compose POW event
    let powEvent =
        EventBuilder.textNote("Another reply with POW")
            .tags([Tag.event(textnoteEvent.id)])
            .pow(20)
            .signWithKeys(keys);

    // Compose note with custom timestamp
    let customTimestamp =
        EventBuilder.textNote("Note with custom timestamp")
            .customCreatedAt(Timestamp.fromSecs(12345678))
            .signWithKeys(keys);
}
