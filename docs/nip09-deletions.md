# Managing deletions (NIP-09)

NIP-09 defines a best-effort mechanism for retracting events you previously published. It does **not** guarantee erasure—relays and downstream clients are allowed to cache or ignore deletion requests—so treat it as a courtesy protocol for well-behaved peers.

## When to send a deletion event

1. You control the secret key that signed the original event.
2. You know the event IDs (or coordinates for replaceable/parameterized events) you want to retract.
3. You accept that the payload remains public even after the deletion request propagates.

## Building the request

```rust
use nostr::prelude::*;

let delete = EventDeletionRequest::new()
    .id(EventId::from_hex("7469af3be8c8e06e1b50ef1caceba30392ddc0b6614507398b7d7daa4c218e96")?)
    // optionally add coordinates for replaceable events
    // .coordinate(Coordinate::parse("30023:pubkey:identifier")?)
    .reason("these posts were published by accident");

let event = EventBuilder::delete(delete).sign_with_keys(&keys)?;
```

Use `Tag::event` for concrete IDs (`e` tags) and `Tag::coordinate` for replaceable events (`a` tags). The textual `reason` is optional but helps other clients explain why the content disappeared.

## Broadcasting and follow-up

1. Send the deletion event to every relay that received the original event.
2. Keep a local record of the IDs you attempted to delete. Some relays respond with `OK` messages or emit `NOTICE`s when they refuse to honor the request; handle both cases.
3. Be prepared for race conditions—if someone republishes the original content, you may have to re-issue a deletion.

## Caveats

- Relays are free to ignore deletion events entirely or only apply them to new subscribers.
- Archive relays and scrapers can continue serving the old content indefinitely.
- Deleting a parameterized replaceable event without its coordinate will have no effect.

In short: use NIP-09 as part of your UX, but do not treat it as a hard delete.
