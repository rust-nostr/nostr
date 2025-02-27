# Leverages Python-Mnemonic package (ref implementation of BIP39) https://github.com/trezor/python-mnemonic
from mnemonic import Mnemonic
from nostr_sdk import Keys

def nip06():
    print()
    # ANCHOR: keys-from-seed24
    # Generate random Seed Phrase (24 words e.g. 256 bits entropy)
    print("Keys from 24 word Seed Phrase:")
    words = Mnemonic("english").generate(strength=256)
    passphrase = ""

    # Use Seed Phrase to generate basic Nostr keys
    keys = Keys.from_mnemonic(words, passphrase)

    print(f" Seed Words (24)  : {words}")
    print(f" Public key bech32: {keys.public_key().to_bech32()}")
    print(f" Secret key bech32: {keys.secret_key().to_bech32()}")
    # ANCHOR_END: keys-from-seed24

    print()
    # ANCHOR: keys-from-seed12
    # Generate random Seed Phrase (12 words e.g. 128 bits entropy)
    print("Keys from 12 word Seed Phrase:")
    words = Mnemonic("english").generate(strength=128)
    passphrase = ""

    # Use Seed Phrase to generate basic Nostr keys
    keys = Keys.from_mnemonic(words, passphrase)

    print(f" Seed Words (12)  : {words}")
    print(f" Public key bech32: {keys.public_key().to_bech32()}")
    print(f" Secret key bech32: {keys.secret_key().to_bech32()}")
    # ANCHOR_END: keys-from-seed12

    print()
    # ANCHOR: keys-from-seed-accounts
    # Advanced (with accounts) from the example wordlist
    words = "leader monkey parrot ring guide accident before fence cannon height naive bean"
    passphrase = ""

    print("Accounts (0-5) from 12 word Seed Phrase (with passphrase):")
    print(f" Seed Words (12): {words}")
    print(" Accounts (0-5) :")

    # Use Seed Phrase and account to multiple Nostr keys
    for account in range(0,6):
        nsec = Keys.from_mnemonic(words, passphrase, account).secret_key().to_bech32()
        print(f"     Account #{account} bech32: {nsec}")
    # ANCHOR_END: keys-from-seed-accounts

    print()
    # ANCHOR: keys-from-seed-accounts-pass
    # Advanced (with accounts) from the same wordlist with in inclusion of passphrase
    words = "leader monkey parrot ring guide accident before fence cannon height naive bean"
    passphrase = "RustNostr"
    print("Accounts (0-5) from 12 word Seed Phrase (with passphrase):")
    print(f" Seed Words (12): {words}")
    print(f" Passphrase     : {passphrase}")
    print(" Accounts (0-5) :")

    # Use Seed Phrase and account to multiple Nostr keys
    for account in range(0,6):
        nsec = Keys.from_mnemonic(words, passphrase, account).secret_key().to_bech32()
        print(f"     Account #{account} bech32: {nsec}")
    # ANCHOR_END: keys-from-seed-accounts-pass

if __name__ == '__main__':
   nip06()