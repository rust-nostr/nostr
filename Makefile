.PHONY: book precommit check

cli:
	cargo build -p nostr-cli --release

precommit: fmt check-crates check-bindings check-docs

fmt:
	@rustup install nightly-2024-01-11
	@rustup component add rustfmt --toolchain nightly-2024-01-11
	cargo +nightly-2024-01-11 fmt --all -- --config format_code_in_doc_comments=true
	cd bindings/nostr-js && make fmt
	cd bindings/nostr-sdk-js && make fmt

check: fmt check-crates check-crates-msrv check-bindings check-docs

check-fmt:
	@rustup install nightly-2024-01-11
	@rustup component add rustfmt --toolchain nightly-2024-01-11
	cargo +nightly-2024-01-11 fmt --all -- --config format_code_in_doc_comments=true --check
	cd bindings/nostr-js && make check-fmt
	cd bindings/nostr-sdk-js && make check-fmt

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