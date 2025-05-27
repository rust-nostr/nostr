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
    let welcome = bob_nostr_mls.process_welcome(&EventId::all_zeros(), &welcome_rumor)?;

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
    let bob_process_result = bob_nostr_mls.process_message(&message_event)?;
    tracing::info!("Bob process_message result: {:?}", bob_process_result);
    tracing::info!("Bob process_message - message: {:?}", bob_process_result.message);
    tracing::info!("Bob process_message - member_changes: {:?}", bob_process_result.member_changes);

    let message = bob_process_result.message.unwrap();
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
        bob_nostr_mls.get_groups().unwrap().len(),
        1,
        "Bob should have 1 group"
    );

    tracing::info!("Alice about to process message");
    let alice_process_result = alice_nostr_mls.process_message(&message_event)?;
    tracing::info!("Alice process_message result: {:?}", alice_process_result);
    tracing::info!("Alice process_message - message: {:?}", alice_process_result.message);
    tracing::info!("Alice process_message - member_changes: {:?}", alice_process_result.member_changes);

    // ================================
    // Testing add_members functionality
    // ================================
    
    tracing::info!("Starting add_members functionality test");
    
    // Generate Charlie's identity as the third user
    let (charlie_keys, charlie_nostr_mls, charlie_temp_dir) = generate_identity();
    tracing::info!("Charlie identity generated");
    
    // Charlie creates key package
    let (charlie_key_package_encoded, charlie_tags) =
        charlie_nostr_mls.create_key_package_for_event(&charlie_keys.public_key(), [relay_url.clone()])?;

    let charlie_key_package_event = EventBuilder::new(Kind::MlsKeyPackage, charlie_key_package_encoded)
        .tags(charlie_tags)
        .build(charlie_keys.public_key())
        .sign(&charlie_keys)
        .await?;
    
    // Alice parses Charlie's key package
    let charlie_key_package: KeyPackage = alice_nostr_mls.parse_key_package(&charlie_key_package_event)?;
    
    // Verify group state before adding members
    let members_before = alice_nostr_mls
        .get_members(&GroupId::from_slice(alice_group.mls_group_id.as_slice()))
        .unwrap();
    tracing::info!("Group has {} members before adding", members_before.len());
    assert_eq!(members_before.len(), 2, "Should have 2 members before adding");

    let secret: group_types::GroupExporterSecret = alice_nostr_mls.exporter_secret(&alice_group.mls_group_id)?;

    
    // Alice uses add_members to add Charlie to the group
    let add_members_result = alice_nostr_mls.add_members(
        &GroupId::from_slice(alice_group.mls_group_id.as_slice()),
        &[charlie_key_package],
    )?;
    
    tracing::info!("Charlie has been added to the group");
    
    // Verify group state after adding members
    let members_after = alice_nostr_mls
        .get_members(&GroupId::from_slice(alice_group.mls_group_id.as_slice()))
        .unwrap();
    tracing::info!("Group has {} members after adding", members_after.len());
    assert_eq!(members_after.len(), 3, "Should have 3 members after adding");
    assert!(members_after.contains(&charlie_keys.public_key()), "Group should contain Charlie's public key");
    
    // Create welcome rumor event for Charlie
    let charlie_welcome_rumor = EventBuilder::new(
        Kind::MlsWelcome, 
        hex::encode(&add_members_result.welcome_message)
    )
    .build(alice_keys.public_key());
    
    // Charlie processes the welcome message
    let charlie_welcome = charlie_nostr_mls.process_welcome(&EventId::all_zeros(), &charlie_welcome_rumor)?;
    
    tracing::debug!("Charlie's Welcome message: {:?}", charlie_welcome);
    
    // Verify welcome message content
    assert_eq!(
        charlie_welcome.member_count as usize, 3,
        "Charlie's welcome message should show group has 3 members"
    );
    assert_eq!(
        charlie_welcome.group_name, "Bob & Alice",
        "Charlie's welcome message should show correct group name"
    );
    
    // Charlie accepts welcome and joins the group
    let charlie_groups = charlie_nostr_mls.get_groups()?;
    let charlie_group = charlie_groups.first().unwrap();
    let charlie_mls_group_id = GroupId::from_slice(charlie_group.mls_group_id.as_slice());
    
    tracing::info!("Charlie has successfully joined the group");
    
    // Verify Charlie's group state
    let charlie_members = charlie_nostr_mls.get_members(&charlie_mls_group_id).unwrap();
    assert_eq!(charlie_members.len(), 3, "Charlie should see 3 members in the group");
    assert!(charlie_members.contains(&alice_keys.public_key()), "Charlie should see Alice in the group");
    assert!(charlie_members.contains(&bob_keys.public_key()), "Charlie should see Bob in the group");
    assert!(charlie_members.contains(&charlie_keys.public_key()), "Charlie should see himself in the group");
    
    // Bob also needs to process the commit message to update group state
    // Use create_commit_proposal_message to create commit event for Bob
    let commit_event = alice_nostr_mls.create_commit_proposal_message(
        &GroupId::from_slice(alice_group.mls_group_id.as_slice()),
        &add_members_result.commit_message,
        &secret.secret
    )?;
        
    // Bob processes the commit message
    let bob_commit_process_result = bob_nostr_mls.process_message(&commit_event)?;
    tracing::info!("Bob process_message (add commit) result: {:?}", bob_commit_process_result);
    tracing::info!("Bob process_message (add commit) - message: {:?}", bob_commit_process_result.message);
    tracing::info!("Bob process_message (add commit) - member_changes: {:?}", bob_commit_process_result.member_changes);
    
    // Verify Bob's group state has been updated
    let bob_members_updated = bob_nostr_mls.get_members(&bob_mls_group_id).unwrap();
    assert_eq!(bob_members_updated.len(), 3, "Bob should see 3 members in the group");
    assert!(bob_members_updated.contains(&charlie_keys.public_key()), "Bob should see Charlie in the group");
    
    tracing::info!("add_members functionality test completed!");
    tracing::info!("Group now has {} members: Alice, Bob, Charlie", bob_members_updated.len());

    // ================================
    // Testing remove_members functionality
    // ================================
    
    tracing::info!("Starting remove_members functionality test");
    
    // Verify group state before removing members
    let members_before_remove = alice_nostr_mls
        .get_members(&GroupId::from_slice(alice_group.mls_group_id.as_slice()))
        .unwrap();
    tracing::info!("Group has {} members before removing", members_before_remove.len());
    assert_eq!(members_before_remove.len(), 3, "Should have 3 members before removing");
    assert!(members_before_remove.contains(&charlie_keys.public_key()), "Group should contain Charlie before removing");

    // Get the current exporter secret before removal
    let secret_before_remove: group_types::GroupExporterSecret = alice_nostr_mls.exporter_secret(&alice_group.mls_group_id)?;
    
    // Alice removes Charlie from the group using hex string format
    let charlie_pubkey_hex = charlie_keys.public_key().to_hex();
    let remove_result = alice_nostr_mls.remove_members(
        &GroupId::from_slice(alice_group.mls_group_id.as_slice()),
        &[charlie_pubkey_hex],
    )?;
    
    tracing::info!("Charlie has been removed from the group");
    
    // Verify group state after removing members
    let members_after_remove = alice_nostr_mls
        .get_members(&GroupId::from_slice(alice_group.mls_group_id.as_slice()))
        .unwrap();
    tracing::info!("Group has {} members after removing", members_after_remove.len());
    assert_eq!(members_after_remove.len(), 2, "Should have 2 members after removing");
    assert!(!members_after_remove.contains(&charlie_keys.public_key()), "Group should not contain Charlie after removing");
    assert!(members_after_remove.contains(&alice_keys.public_key()), "Group should still contain Alice");
    assert!(members_after_remove.contains(&bob_keys.public_key()), "Group should still contain Bob");
    
    // Create commit event for the removal
    let remove_commit_event = alice_nostr_mls.create_commit_proposal_message(
        &GroupId::from_slice(alice_group.mls_group_id.as_slice()),
        &remove_result.serialized,
        &secret_before_remove.secret
    )?;
    
    // Bob processes the removal commit message
    let bob_remove_process_result = bob_nostr_mls.process_message(&remove_commit_event)?;
    tracing::info!("Bob process_message (remove commit) result: {:?}", bob_remove_process_result);
    tracing::info!("Bob process_message (remove commit) - message: {:?}", bob_remove_process_result.message);
    tracing::info!("Bob process_message (remove commit) - member_changes: {:?}", bob_remove_process_result.member_changes);
    
    // Verify Bob's group state has been updated
    let bob_members_after_remove = bob_nostr_mls.get_members(&bob_mls_group_id).unwrap();
    assert_eq!(bob_members_after_remove.len(), 2, "Bob should see 2 members in the group after removal");
    assert!(!bob_members_after_remove.contains(&charlie_keys.public_key()), "Bob should not see Charlie in the group");
    assert!(bob_members_after_remove.contains(&alice_keys.public_key()), "Bob should still see Alice in the group");
    assert!(bob_members_after_remove.contains(&bob_keys.public_key()), "Bob should still see himself in the group");
    
    tracing::info!("remove_members functionality test completed!");
    tracing::info!("Group now has {} members: Alice, Bob", bob_members_after_remove.len());

    // ================================
    // Testing leave_group functionality
    // ================================
    
    tracing::info!("Starting leave_group functionality test");
    
    // Verify group state before Bob leaves
    let alice_members_before_leave = alice_nostr_mls
        .get_members(&GroupId::from_slice(alice_group.mls_group_id.as_slice()))
        .unwrap();
    let bob_members_before_leave = bob_nostr_mls.get_members(&bob_mls_group_id).unwrap();
    
    tracing::info!("Alice sees {} members before Bob leaves", alice_members_before_leave.len());
    tracing::info!("Bob sees {} members before leaving", bob_members_before_leave.len());
    assert_eq!(alice_members_before_leave.len(), 2, "Alice should see 2 members before Bob leaves");
    assert_eq!(bob_members_before_leave.len(), 2, "Bob should see 2 members before leaving");
    assert!(alice_members_before_leave.contains(&bob_keys.public_key()), "Alice should see Bob in the group before he leaves");
    assert!(bob_members_before_leave.contains(&bob_keys.public_key()), "Bob should see himself in the group before leaving");

    // Get the current exporter secret before Bob leaves
    let secret_before_leave: group_types::GroupExporterSecret = bob_nostr_mls.exporter_secret(&bobs_group.mls_group_id)?;
    
    // Bob leaves the group
    let leave_result = bob_nostr_mls.leave_group(&bob_mls_group_id)?;
    
    tracing::info!("Bob has left the group");
    
    // Verify Bob's local state after leaving
    let bob_groups_after_leave = bob_nostr_mls.get_groups().unwrap();
    tracing::info!("Bob has {} groups after leaving", bob_groups_after_leave.len());
    // Note: The group might still exist locally but Bob should no longer be a member
    
    // Create commit event for Bob's departure
    let leave_commit_event = bob_nostr_mls.create_commit_proposal_message(
        &bob_mls_group_id,
        &leave_result.serialized,
        &secret_before_leave.secret
    )?;
    
    // Alice processes Bob's leave commit message
    let alice_leave_process_result = alice_nostr_mls.process_message(&leave_commit_event)?;
    tracing::info!("Alice process_message (leave commit) result: {:?}", alice_leave_process_result);
    tracing::info!("Alice process_message (leave commit) - message: {:?}", alice_leave_process_result.message);
    tracing::info!("Alice process_message (leave commit) - member_changes: {:?}", alice_leave_process_result.member_changes);
    
    // Verify Alice's group state has been updated
    let alice_members_after_leave = alice_nostr_mls
        .get_members(&GroupId::from_slice(alice_group.mls_group_id.as_slice()))
        .unwrap();
    tracing::info!("Alice sees {} members after Bob leaves", alice_members_after_leave.len());
    assert_eq!(alice_members_after_leave.len(), 1, "Alice should see 1 member in the group after Bob leaves");
    assert!(!alice_members_after_leave.contains(&bob_keys.public_key()), "Alice should not see Bob in the group after he leaves");
    assert!(alice_members_after_leave.contains(&alice_keys.public_key()), "Alice should still see herself in the group");
    
    tracing::info!("leave_group functionality test completed!");
    tracing::info!("Group now has {} member: Alice", alice_members_after_leave.len());

    cleanup(alice_temp_dir, bob_temp_dir);
    charlie_temp_dir
        .close()
        .expect("Failed to close Charlie's temp directory");

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
