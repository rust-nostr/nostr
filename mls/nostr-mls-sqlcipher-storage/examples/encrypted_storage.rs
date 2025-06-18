//! Example demonstrating SQLCipher encryption features
//! 
//! This example shows how to:
//! - Create an encrypted database
//! - Open an encrypted database with a password
//! - Change the password of an encrypted database
//! - Handle encryption-related errors

use std::env;
use nostr_mls_sqlcipher_storage::{NostrMlsSqliteStorage, error::Error};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SQLCipher Encryption Example");
    println!("============================");

    // Example 1: Create an encrypted database
    println!("\n1. Creating an encrypted database...");
    let password = "my_secret_password_123";
    let storage = NostrMlsSqliteStorage::new_with_password("encrypted_example.db", Some(password))?;
    println!("✓ Encrypted database created successfully");

    // Example 2: Check if database is encrypted (this will always return false for an open connection with correct password)
    match storage.is_encrypted() {
        Ok(encrypted) => println!("✓ Database encryption status: {}", if encrypted { "encrypted" } else { "accessible (correct password or unencrypted)" }),
        Err(e) => println!("⚠ Could not check encryption status: {}", e),
    }

    // Example 3: Change password
    println!("\n2. Changing database password...");
    let new_password = "new_secret_password_456";
    storage.change_password(Some(new_password))?;
    println!("✓ Password changed successfully");

    // Close the current connection
    drop(storage);

    // Example 4: Try to open with old password (should fail)
    println!("\n3. Testing old password (should fail)...");
    match NostrMlsSqliteStorage::new_with_password("encrypted_example.db", Some(password)) {
        Ok(_) => println!("⚠ Unexpected: Old password still works"),
        Err(e) => println!("✓ Expected: Old password rejected - {}", e),
    }

    // Example 5: Open with new password (should work)
    println!("\n4. Opening with new password...");
    let storage = NostrMlsSqliteStorage::new_with_password("encrypted_example.db", Some(new_password))?;
    println!("✓ Successfully opened with new password");

    // Example 6: Try to remove encryption (will fail - not supported)
    println!("\n5. Attempting to remove encryption (should fail)...");
    match storage.change_password(None) {
        Ok(_) => println!("⚠ Unexpected: Encryption removal succeeded"),
        Err(Error::Database(msg)) if msg.contains("not currently supported") => {
            println!("✓ Expected: Encryption removal not supported - {}", msg);
        }
        Err(e) => println!("⚠ Unexpected error: {}", e),
    }

    // Example 7: Environment-based password
    println!("\n6. Environment-based password example...");
    env::set_var("DATABASE_PASSWORD", "env_password_789");
    let env_password = env::var("DATABASE_PASSWORD").ok();
    let env_storage = NostrMlsSqliteStorage::new_with_password(
        "env_encrypted_example.db", 
        env_password.as_deref()
    )?;
    println!("✓ Database created with environment-based password");

    // Clean up
    drop(storage);
    drop(env_storage);

    // Clean up files
    let _ = std::fs::remove_file("encrypted_example.db");
    let _ = std::fs::remove_file("env_encrypted_example.db");

    println!("\n✓ Example completed successfully!");
    println!("\nKey takeaways:");
    println!("- Use `new_with_password()` to create encrypted databases");
    println!("- Use `change_password()` to change encryption passwords");
    println!("- Store passwords securely (environment variables, secure vaults, etc.)");
    println!("- Removing encryption is not currently supported");

    Ok(())
} 