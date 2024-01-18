.PHONY: book

cli:
	cargo build -p nostr-cli --release

precommit: fmt check-crates check-bindings check-docs

fmt:
	@rustup install nightly-2024-01-11
	@rustup component add rustfmt --toolchain nightly-2024-01-11
	cargo +nightly-2024-01-11 fmt --all -- --config format_code_in_doc_comments=true

check: fmt check-crates check-crates-msrv check-bindings check-docs

check-fmt:
	@rustup install nightly-2024-01-11
	@rustup component add rustfmt --toolchain nightly-2024-01-11
	cargo +nightly-2024-01-11 fmt --all -- --config format_code_in_doc_comments=true --check

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

indexes-perf:
	cd crates/nostr-database/fuzz/perf && make graph

clean:
	cargo clean

book:
	cd book && just serve

flatbuf:
	cd crates/nostr-database && make flatbuf

loc:
	@echo "--- Counting lines of .rs files (LOC):" && find crates/ bindings/ -type f -name "*.rs" -exec cat {} \; | wc -l