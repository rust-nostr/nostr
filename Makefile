.PHONY: book

precommit:
	@bash .githooks/pre-push

bench:
	RUSTFLAGS='--cfg=bench' cargo +nightly bench -p nostr

indexes-perf:
	cd crates/nostr-database/fuzz/perf && make graph

clean:
	cargo clean

book:
	cd book && make build

flatbuf:
	cd crates/nostr-database && make flatbuf

loc:
	@echo "--- Counting lines of .rs files (LOC):" && find crates/ bindings/ -type f -name "*.rs" -exec cat {} \; | wc -l