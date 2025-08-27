// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use aes_gcm::aead::OsRng;
use aes_gcm::{Aes128Gcm, KeyInit};
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

pub fn generate_encryption_key() -> Vec<u8> {
    Aes128Gcm::generate_key(OsRng).to_vec()
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
    let _bob_key_package: KeyPackage = alice_nostr_mls.parse_key_package(&bob_key_package_event)?;

    let image_url = "http://blossom_server:4531/fake_img.png".to_owned();
    let image_key = generate_encryption_key();
    let name = "Bob & Alice".to_owned();
    let description = "A secret chat between Bob and Alice".to_owned();

    let config = NostrGroupConfigData::new(
        name,
        description,
        Some(image_url),
        Some(image_key),
        vec![relay_url.clone()],
        vec![alice_keys.public_key(), bob_keys.public_key()],
    );

    // Alice creates the group, adding Bob.
    let group_create_result = alice_nostr_mls.create_group(
        &alice_keys.public_key(),
        vec![bob_key_package_event.clone()],
        config,
    )?;

    tracing::info!("Group created");

    // The group is created, and the welcome messages are in welcome_rumors.
    // We also have the Nostr group data, which we can use to show info about the group.
    let alice_group = group_create_result.group;
    let welcome_rumors = group_create_result.welcome_rumors;

    // Alice now creates a Kind: 444 event that is Gift-wrapped to just Bob with the welcome event in the rumor event.
    // If you added multiple users to the group, you'd create a separate gift-wrapped welcome event for each user.
    let welcome_rumor = welcome_rumors
        .first()
        .expect("Should have at least one welcome rumor");

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
    bob_nostr_mls.process_welcome(&EventId::all_zeros(), welcome_rumor)?;
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

    assert_eq!(
        bob_nostr_mls
            .get_groups()
            .unwrap()
            .first()
            .unwrap()
            .nostr_group_id,
        alice_group.nostr_group_id,
        "Bob's group should have the same Nostr group ID as Alice's group"
    );

    assert_eq!(
        hex::encode(
            bob_nostr_mls
                .get_groups()
                .unwrap()
                .first()
                .unwrap()
                .nostr_group_id
        ),
        message_event
            .tags
            .iter()
            .find(|tag| tag.kind() == TagKind::h())
            .unwrap()
            .content()
            .unwrap(),
        "Bob's group should have the same Nostr group ID as Alice's message wrapper event"
    );
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

    tracing::info!("Bob processed message");
    let messages = bob_nostr_mls
        .get_messages(&bob_mls_group_id)
        .map_err(|e| crate::error::Error::Message(e.to_string()))?;
    tracing::info!("Bob got messages: {:?}", messages);
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

    // ================================
    // Extended functionality: Adding Charlie
    // ================================

    let (charlie_keys, charlie_nostr_mls, charlie_temp_dir) = generate_identity();
    tracing::info!("Charlie identity generated");

    // Create key package for Charlie
    let (charlie_key_package_encoded, charlie_tags) = charlie_nostr_mls
        .create_key_package_for_event(&charlie_keys.public_key(), [relay_url.clone()])?;

    let charlie_key_package_event =
        EventBuilder::new(Kind::MlsKeyPackage, charlie_key_package_encoded)
            .tags(charlie_tags)
            .build(charlie_keys.public_key())
            .sign(&charlie_keys)
            .await?;

    // Alice adds Charlie to the group
    tracing::info!("Alice adding Charlie to the group");
    let add_charlie_result = alice_nostr_mls.add_members(
        &GroupId::from_slice(alice_group.mls_group_id.as_slice()),
        &[charlie_key_package_event.clone()],
    )?;

    // Alice publishes the add commit message and Bob processes it
    tracing::info!("Bob processing Charlie addition commit");
    let add_commit_result = bob_nostr_mls.process_message(&add_charlie_result.evolution_event);
    tracing::info!("Add commit processing result: {:?}", add_commit_result);

    // Alice merges the pending commit for adding Charlie
    alice_nostr_mls
        .merge_pending_commit(&GroupId::from_slice(alice_group.mls_group_id.as_slice()))?;

    // Charlie processes the welcome message
    if let Some(welcome_rumors) = add_charlie_result.welcome_rumors {
        let charlie_welcome_rumor = welcome_rumors
            .first()
            .expect("Should have welcome rumor for Charlie");
        charlie_nostr_mls.process_welcome(&EventId::all_zeros(), charlie_welcome_rumor)?;

        let charlie_welcomes = charlie_nostr_mls
            .get_pending_welcomes()
            .expect("Error getting Charlie's pending welcomes");
        let charlie_welcome = charlie_welcomes.first().unwrap();
        charlie_nostr_mls.accept_welcome(charlie_welcome)?;

        tracing::info!("Charlie joined the group");

        // Verify Charlie is in the group
        let group_members = alice_nostr_mls
            .get_members(&GroupId::from_slice(alice_group.mls_group_id.as_slice()))?;
        assert_eq!(group_members.len(), 3, "Group should now have 3 members");
        assert!(
            group_members.contains(&charlie_keys.public_key()),
            "Charlie should be in the group"
        );
    }

    // ================================
    // Removing Charlie from the group
    // ================================

    tracing::info!("Alice removing Charlie from the group");
    let remove_charlie_result = alice_nostr_mls.remove_members(
        &GroupId::from_slice(alice_group.mls_group_id.as_slice()),
        &[charlie_keys.public_key()],
    )?;

    // Bob processes the remove commit message
    tracing::info!("Bob processing Charlie removal commit");
    let remove_commit_result =
        bob_nostr_mls.process_message(&remove_charlie_result.evolution_event);
    tracing::info!(
        "Remove commit processing result: {:?}",
        remove_commit_result
    );

    // Alice merges the pending commit for removing Charlie
    alice_nostr_mls
        .merge_pending_commit(&GroupId::from_slice(alice_group.mls_group_id.as_slice()))?;

    // Verify Charlie is no longer in the group
    let group_members_after_removal =
        alice_nostr_mls.get_members(&GroupId::from_slice(alice_group.mls_group_id.as_slice()))?;
    assert_eq!(
        group_members_after_removal.len(),
        2,
        "Group should now have 2 members"
    );
    assert!(
        !group_members_after_removal.contains(&charlie_keys.public_key()),
        "Charlie should not be in the group"
    );

    // ================================
    // Bob leaving the group
    // ================================

    tracing::info!("Bob leaving the group");
    let bob_leave_result =
        bob_nostr_mls.leave_group(&GroupId::from_slice(bobs_group.mls_group_id.as_slice()))?;

    // Alice processes Bob's leave proposal
    tracing::info!("Alice processing Bob's leave proposal");
    let leave_proposal_result = alice_nostr_mls.process_message(&bob_leave_result.evolution_event);
    tracing::info!(
        "Leave proposal processing result: {:?}",
        leave_proposal_result
    );

    // The leave creates a proposal that needs to be committed by an admin (Alice)
    // Alice should create a commit to finalize Bob's removal
    // Note: In a real application, Alice would need to detect the proposal and create a commit
    // For now, we'll verify the proposal was processed correctly

    match leave_proposal_result {
        Ok(MessageProcessingResult::Proposal(_)) => {
            tracing::info!("Bob's leave proposal was successfully processed by Alice");
        }
        _ => {
            tracing::warn!("Unexpected result from processing Bob's leave proposal");
        }
    }

    tracing::info!("MLS group operations completed successfully!");

    cleanup(alice_temp_dir, bob_temp_dir, charlie_temp_dir);

    Ok(())
}

fn cleanup(alice_temp_dir: TempDir, bob_temp_dir: TempDir, charlie_temp_dir: TempDir) {
    alice_temp_dir
        .close()
        .expect("Failed to close temp directory");
    bob_temp_dir
        .close()
        .expect("Failed to close temp directory");
    charlie_temp_dir
        .close()
        .expect("Failed to close temp directory");
}
