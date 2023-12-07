from nostr_protocol import NostrLibrary

hash = NostrLibrary().git_hash_version()
print(hash)