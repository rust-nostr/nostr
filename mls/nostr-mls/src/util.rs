use nostr::nips::nip44;
use nostr::{Keys, SecretKey};
use nostr_mls_storage::groups::types::GroupExporterSecret;

use crate::Error;

pub(crate) fn decrypt_with_exporter_secret(
    secret: &GroupExporterSecret,
    encrypted_content: &str,
) -> Result<Vec<u8>, Error> {
    // Convert that secret to nostr keys
    let secret_key: SecretKey = SecretKey::from_slice(&secret.secret)?;
    let export_nostr_keys = Keys::new(secret_key);

    // Decrypt message
    let message_bytes: Vec<u8> = nip44::decrypt_to_bytes(
        export_nostr_keys.secret_key(),
        &export_nostr_keys.public_key,
        encrypted_content,
    )?;

    Ok(message_bytes)
}
