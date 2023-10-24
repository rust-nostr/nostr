// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Nostr SDK Flatbuffers

use event_generated::event_fbs::{Fixed32Bytes, Fixed64Bytes};
pub use flatbuffers::FlatBufferBuilder;
use nostr::Event;

#[allow(unused_imports, dead_code)]
mod event_generated;

pub use self::event_generated::event_fbs;

pub fn serialize_event<'a>(fbb: &'a mut FlatBufferBuilder, event: &Event) -> &'a [u8] {
    fbb.reset();

    let id = Fixed32Bytes::new(&event.id.to_bytes());
    let pubkey = Fixed32Bytes::new(&event.pubkey.serialize());
    let sig = Fixed64Bytes::new(event.sig.as_ref());
    let args = event_fbs::EventArgs {
        id: Some(&id),
        pubkey: Some(&pubkey),
        created_at: event.created_at.as_u64(),
        kind: event.kind.as_u64(),
        tags: None,    // TODO
        content: None, // TODO
        sig: Some(&sig),
    };

    let offset = event_fbs::Event::create(fbb, &args);

    event_fbs::finish_event_buffer(fbb, offset);

    fbb.finished_data()
}
