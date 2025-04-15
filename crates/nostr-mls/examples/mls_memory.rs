// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_mls::prelude::*;
use nostr_mls_memory_storage::NostrMlsMemoryStorage;

fn generate_identity() -> (Keys, NostrMls<NostrMlsMemoryStorage>) {
    let keys = Keys::generate();
    let nostr_mls = NostrMls::new(NostrMlsMemoryStorage::default());
    (keys, nostr_mls)
}

#[tokio::main]
async fn main() -> Result<()> {
    let relay_url = RelayUrl::parse("ws://localhost:8080").unwrap();

    let (alice_keys, alice_nostr_mls) = generate_identity();
    let (bob_keys, bob_nostr_mls) = generate_identity();

    // Create key package for Bob
    let bob_key_package_event: Event = bob_nostr_mls
        .create_key_package(&bob_keys, [relay_url.clone()])
        .await?;

    // ================================
    // We're now acting as Alice
    // ================================

    // To create a group, Alice fetches Bob's key package from the Nostr network and parses it
    let bob_key_package: KeyPackage =
        alice_nostr_mls.parse_key_package_event(&bob_key_package_event)?;

    // Alice creates the group, adding Bob.
    let group_create_result = alice_nostr_mls.create_group(
        "Bob & Alice",
        "A secret chat between Bob and Alice",
        &alice_keys.public_key,
        vec![bob_key_package],
        vec![alice_keys.public_key(), bob_keys.public_key()],
        vec![RelayUrl::parse("ws://localhost:8080").unwrap()],
    )?;

    // The group is created, and the welcome message is serialized to send to Bob.
    // We also have the Nostr group data, which we can use to show info about the group.
    let alice_mls_group = group_create_result.mls_group;
    let serialized_welcome_message = group_create_result.serialized_welcome_message;
    let alice_group_data = group_create_result.nostr_group_data;

    // At this point, Alice would publish a Kind: 444 event that is Gift-wrapped to just
    // Bob with the welcome event in the rumor event.

    // Now, let's also try sending a message to the group (using an unsigned Kind: 9 event)
    // We don't have to wait for Bob to join the group before we send our first message.
    let rumor = EventBuilder::new(Kind::Custom(9), "Hi Bob!").build(alice_keys.public_key());

    // Get the export secret value for this epoch of the group
    // In real usage you would want to do this once per epoch, per group, and cache it.
    // 🚨 It's critical that you delete this secret after some period of time to preserve forward secrecy.
    // For example, once the group has moved 2 epochs beyond this one.
    let CreateMessage {
        event: message_event,
        secret,
    } = alice_nostr_mls.create_message(
        alice_mls_group.group_id(),
        alice_group_data.nostr_group_id,
        rumor,
    )?;

    // ================================
    // We're now acting as Bob
    // ================================

    // First Bob recieves the Gift-wrapped welcome message from Alice and decrypts it.
    // Bob can now preview the welcome message to see what group he might be joining
    let welcome_preview = bob_nostr_mls
        .preview_welcome_event(serialized_welcome_message.clone())
        .expect("Error previewing welcome event");
    assert_eq!(
        welcome_preview.staged_welcome.members().count(),
        alice_mls_group.members().count(),
        "Welcome message group member count should match the group member count"
    );
    assert_eq!(
        welcome_preview.nostr_group_data.name, "Bob & Alice",
        "Welcome message group name should be Bob & Alice"
    );

    // Bob can now join the group
    let join_result = bob_nostr_mls.join_group_from_welcome(serialized_welcome_message.clone())?;
    let bob_mls_group = join_result.mls_group;
    let bob_group_data = join_result.nostr_group_data;

    // Bob and Alice now have synced state for the group.
    assert_eq!(
        bob_mls_group.members().count(),
        alice_mls_group.members().count(),
        "Groups should have 2 members"
    );
    assert_eq!(
        bob_group_data.name, "Bob & Alice",
        "Group name should be Bob & Alice"
    );

    // The resulting serialized message is the MLS encrypted message that Bob sent
    // Now Bob can process the MLS message content and do what's needed with it
    let rumor = bob_nostr_mls
        .process_message(bob_mls_group.group_id(), secret, &message_event)?
        .unwrap();

    assert_eq!(
        rumor.kind,
        Kind::Custom(9),
        "Message event kind should be Custom(9)"
    );
    assert_eq!(
        rumor.pubkey,
        alice_keys.public_key(),
        "Message event pubkey should be Alice's pubkey"
    );
    assert_eq!(
        rumor.content, "Hi Bob!",
        "Message event content should be Hi Bob!"
    );

    Ok(())
}
