# ANCHOR: full
from nostr_sdk import SecretKey, EncryptedSecretKey, KeySecurity

def encrypt():
    # ANCHOR: parse-secret-key
    secret_key = SecretKey.parse("3501454135014541350145413501453fefb02227e449e57cf4d3a3ce05378683")
    # ANCHOR_END: parse-secret-key

    # ANCHOR: encrypt-default
    password = "nostr"
    encrypted = secret_key.encrypt(password)
    # ANCHOR_END: encrypt-default

    print(f"Encrypted secret key: {encrypted.to_bech32()}")

    # ANCHOR: encrypt-custom
    encrypted_custom = EncryptedSecretKey(secret_key, password, 12, KeySecurity.WEAK)
    # ANCHOR_END: encrypt-custom

    print(f"Encrypted secret key (custom): {encrypted_custom.to_bech32()}")

def decrypt():
    # ANCHOR: parse-ncryptsec
    encrypted = EncryptedSecretKey.from_bech32("ncryptsec1qgg9947rlpvqu76pj5ecreduf9jxhselq2nae2kghhvd5g7dgjtcxfqtd67p9m0w57lspw8gsq6yphnm8623nsl8xn9j4jdzz84zm3frztj3z7s35vpzmqf6ksu8r89qk5z2zxfmu5gv8th8wclt0h4p")
    # ANCHOR_END: parse-ncryptsec

    # ANCHOR: decrypt
    secret_key = encrypted.to_secret_key("nostr")
    # ANCHOR_END: decrypt

    print(f"Decrypted secret key: {secret_key.to_bech32()}")


if __name__ == '__main__':
   encrypt()
   decrypt()
# ANCHOR_END: full
