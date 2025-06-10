// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_mls::prelude::*;
use nostr_mls_sqlite_storage::NostrMlsSqliteStorage;
use tempfile::TempDir;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

/// Generate a new identity and return the keys, NostrMls instance, and temp directory
/// We use a different temp directory for each identity because OpenMLS doesn't have a concept of partitioning storage for different identities.
/// Because of this, we need to create diffrent databases for each identity.
fn generate_identity() -> (Keys, NostrMls<NostrMlsSqliteStorage>, TempDir) {
    let keys = Keys::generate();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("mls.db");
    let nostr_mls = NostrMls::new(NostrMlsSqliteStorage::new(db_path).unwrap());
    (keys, nostr_mls, temp_dir)
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let relay_url = RelayUrl::parse("ws://localhost:8080").unwrap();

    let (alice_keys, alice_nostr_mls, alice_temp_dir) = generate_identity();
    tracing::info!("Alice identity generated");
    let (bob_keys, bob_nostr_mls, bob_temp_dir) = generate_identity();
    tracing::info!("Bob identity generated");

    // Create key package for Bob
    // This would be published to the Nostr network for other users to find
    let (bob_key_package_encoded, tags) =
        bob_nostr_mls.create_key_package_for_event(&bob_keys.public_key(), [relay_url.clone()])?;

    let bob_key_package_event = EventBuilder::new(Kind::MlsKeyPackage, bob_key_package_encoded)
        .tags(tags)
        .build(bob_keys.public_key())
        .sign(&bob_keys)
        .await?;

    // ================================
    // We're now acting as Alice
    // ================================

    // To create a group, Alice fetches Bob's key package from the Nostr network and parses it
    let bob_key_package: KeyPackage = alice_nostr_mls.parse_key_package(&bob_key_package_event)?;

    // Alice creates the group, adding Bob.
    let group_create_result = alice_nostr_mls.create_group(
        "Bob & Alice",
        "A secret chat between Bob and Alice",
        &alice_keys.public_key,
        &[bob_keys.public_key()],
        &[bob_key_package],
        vec![alice_keys.public_key()],
        vec![RelayUrl::parse("ws://localhost:8080").unwrap()],
    )?;

    tracing::info!("Group created");

    // The group is created, and the welcome message is serialized to send to Bob.
    // We also have the Nostr group data, which we can use to show info about the group.
    let alice_group = group_create_result.group;
    let serialized_welcome_message = group_create_result.serialized_welcome_message;

    // Alice now creates a Kind: 444 event that is Gift-wrapped to just Bob with the welcome event in the rumor event.
    // If you added multiple users to the group, you'd create a separate gift-wrapped welcome event (with the same serialized_welcome_message) for each user.
    let welcome_rumor =
        EventBuilder::new(Kind::MlsWelcome, hex::encode(&serialized_welcome_message))
            .tags(vec![
                // These relays are the group_relays where the user should look for group messages.
                // Tag::from_standardized(TagStandard::Relays(
                //     relay_urls
                //         .iter()
                //         .filter_map(|r| RelayUrl::parse(r).ok())
                //         .collect(),
                // )),
                // Tag::event(member.event_id), // This is the event ID of the key_package event used when adding the member to the group
            ])
            .build(alice_keys.public_key());

    // Now, let's also try sending a message to the group (using an unsigned Kind: 9 event)
    // We don't have to wait for Bob to join the group before we send our first message.
    let rumor = EventBuilder::new(Kind::Custom(9), "Hi Bob!").build(alice_keys.public_key());
    let message_event = alice_nostr_mls.create_message(
        &GroupId::from_slice(alice_group.mls_group_id.as_slice()),
        rumor.clone(),
    )?;
    // Alice would now publish the message_event to the Nostr network.
    tracing::info!("Message inner event created: {:?}", rumor);
    tracing::debug!("Message wrapper event created: {:?}", message_event);

    // ================================
    // We're now acting as Bob
    // ================================

    // First Bob recieves the Gift-wrapped welcome message from Alice, decrypts it, and processes it.
    // The first param is the gift-wrap event id (which we set as all zeros for this example)
    bob_nostr_mls.process_welcome(&EventId::all_zeros(), &welcome_rumor)?;
    // Bob can now preview the welcome message to see what group he might be joining
    let welcomes = bob_nostr_mls
        .get_pending_welcomes()
        .expect("Error getting pending welcomes");
    let welcome = welcomes.first().unwrap();

    tracing::debug!("Welcome for Bob: {:?}", welcome);

    assert_eq!(
        welcome.member_count as usize,
        alice_nostr_mls
            .get_members(&GroupId::from_slice(alice_group.mls_group_id.as_slice()))
            .unwrap()
            .len(),
        "Welcome message group member count should match the group member count"
    );
    assert_eq!(
        welcome.group_name, "Bob & Alice",
        "Welcome message group name should be Bob & Alice"
    );

    // Bob can now join the group
    bob_nostr_mls.accept_welcome(welcome)?;
    let bobs_group = bob_nostr_mls.get_groups()?.first().unwrap().clone();
    let bob_mls_group_id = GroupId::from_slice(bobs_group.mls_group_id.as_slice());

    tracing::info!("Bob joined group");

    // Bob and Alice now have synced state for the group.
    assert_eq!(
        bob_nostr_mls.get_members(&bob_mls_group_id).unwrap().len(),
        alice_nostr_mls
            .get_members(&GroupId::from_slice(alice_group.mls_group_id.as_slice()))
            .unwrap()
            .len(),
        "Groups should have 2 members"
    );
    assert_eq!(
        bobs_group.name, "Bob & Alice",
        "Group name should be Bob & Alice"
    );

    tracing::info!("Bob about to process message");

    // The resulting serialized message is the MLS encrypted message that Bob sent
    // Now Bob can process the MLS message content and do what's needed with it
    bob_nostr_mls.process_message(&message_event)?;

    let messages = bob_nostr_mls.get_messages(&bob_mls_group_id).unwrap();
    let message = messages.first().unwrap();
    tracing::info!("Bob processed message: {:?}", message);

    assert_eq!(
        message.kind,
        Kind::Custom(9),
        "Message event kind should be Custom(9)"
    );
    assert_eq!(
        message.pubkey,
        alice_keys.public_key(),
        "Message event pubkey should be Alice's pubkey"
    );
    assert_eq!(
        message.content, "Hi Bob!",
        "Message event content should be Hi Bob!"
    );

    assert_eq!(
        alice_nostr_mls.get_groups().unwrap().len(),
        1,
        "Alice should have 1 group"
    );

    assert_eq!(
        alice_nostr_mls
            .get_messages(&GroupId::from_slice(alice_group.mls_group_id.as_slice()))
            .unwrap()
            .len(),
        1,
        "Alice should have 1 message"
    );

    assert_eq!(
        bob_nostr_mls.get_groups().unwrap().len(),
        1,
        "Bob should have 1 group"
    );

    assert_eq!(
        bob_nostr_mls
            .get_messages(&GroupId::from_slice(bobs_group.mls_group_id.as_slice()))
            .unwrap()
            .len(),
        1,
        "Bob should have 1 message"
    );

    tracing::info!("Alice about to process message");
    alice_nostr_mls.process_message(&message_event)?;

    let messages = alice_nostr_mls
        .get_messages(&GroupId::from_slice(alice_group.mls_group_id.as_slice()))
        .unwrap();
    let message = messages.first().unwrap();
    tracing::info!("Alice processed message: {:?}", message);

    cleanup(alice_temp_dir, bob_temp_dir);

    Ok(())
}

fn cleanup(alice_temp_dir: TempDir, bob_temp_dir: TempDir) {
    alice_temp_dir
        .close()
        .expect("Failed to close temp directory");
    bob_temp_dir
        .close()
        .expect("Failed to close temp directory");
}
