package com.nostr.signer;

interface ISigner {
    String getPublicKey();

    /** Take an unsigned event and returns the signed one. */
    String signEvent(in String unsigned_event);

    /** Encrypt the plaintext using NIP-44 */
    String nip44Encrypt(in String current_user_public_key, in String public_key, in String plaintext);

    /** Decrypt the ciphertext using NIP-44 */
    String nip44Decrypt(in String current_user_public_key, in String public_key, in String ciphertext);
}
