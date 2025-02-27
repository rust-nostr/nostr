from nostr_sdk import Keys, PublicKey, nip44_encrypt, nip44_decrypt, Nip44Version


def nip44():
    print("\nEncrypting and Decrypting Messages (NIP-44):")
    keys = Keys.generate()

    pk = PublicKey.parse("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")

    ciphertext = nip44_encrypt(keys.secret_key(), pk, "my message", Nip44Version.V2)
    print(f" Encrypted: {ciphertext}")

    plaintext = nip44_decrypt(keys.secret_key(), pk, ciphertext)
    print(f" Decrypted: {plaintext}")

if __name__ == '__main__':
   nip44()