.PHONY: book precommit check

cli:
	cargo build -p nostr-cli --release

precommit: fmt check-crates check-bindings check-docs

check: fmt check-crates check-crates-msrv check-bindings check-docs

fmt:
	@bash contrib/scripts/check-fmt.sh

check-fmt:
	@bash contrib/scripts/check-fmt.sh check

check-bindings:
	@bash contrib/scripts/check-bindings.sh

check-book:
	@bash contrib/scripts/check-book.sh

check-crates:
	@bash contrib/scripts/check-crates.sh

check-crates-msrv:
	@bash contrib/scripts/check-crates.sh msrv

check-docs:
	@bash contrib/scripts/check-docs.sh

# Release rust crates
release:
	@bash contrib/scripts/release.sh

bench:
	RUSTFLAGS='--cfg=bench' cargo +nightly bench -p nostr

clean:
	cargo clean
	cd bindings/nostr-js && cargo clean
	cd bindings/nostr-sdk-js && cargo clean

book:
	cd book && just serve

flatbuf:
	cd crates/nostr-database && make flatbuf

loc:
	@echo "--- Counting lines of .rs files (LOC):" && find crates/ bindings/ -type f -name "*.rs" -exec cat {} \; | wc -l