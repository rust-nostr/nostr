from nostr_protocol import NostrLibrary

git_hash = NostrLibrary().git_hash_version()
print(git_hash)
