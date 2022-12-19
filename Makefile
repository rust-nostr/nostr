# Use 'verbose=1' to echo all commands, for example 'make help verbose=1'.
ifdef verbose
  Q :=
else
  Q := @
endif

precommit:
	$(Q)cargo fmt --all -- --config format_code_in_doc_comments=true
	$(Q)cargo clippy --all
	$(Q)cargo clippy -p nostr --no-default-features
	$(Q)cargo clippy -p nostr --features all-nips
	$(Q)cargo clippy -p nostr-sdk --no-default-features
	$(Q)cargo clippy -p nostr-sdk --features all-nips
	$(Q)cargo clippy -p nostr-sdk --features blocking
	$(Q)cargo clippy -p nostr-ffi
	$(Q)cargo clippy -p nostr-sdk-ffi

test:
	$(Q)cargo test --all --all-features

clean:
	$(Q)cargo clean

loc:
	$(Q)echo "--- Counting lines of .rs files (LOC):" && find crates/ -type f -name "*.rs" -exec cat {} \; | wc -l