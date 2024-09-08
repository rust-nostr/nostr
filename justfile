#!/usr/bin/env just --justfile

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Build nostr CLI (release)
cli:
	cargo build -p nostr-cli --release

# Execute a partial check (MSRV is not checked)
precommit:
    @bash contrib/scripts/precommit.sh

# Execute a full check
check:
    @bash contrib/scripts/check.sh

# Format the entire Rust code
fmt:
	@bash contrib/scripts/check-fmt.sh

# Check if the Rust code is formatted
check-fmt:
	@bash contrib/scripts/check-fmt.sh check

# Check all the bindings
check-bindings:
	@bash contrib/scripts/check-bindings.sh

# Check the book snippets
check-book:
	@bash contrib/scripts/check-book.sh

# Check all the crates
check-crates:
	@bash contrib/scripts/check-crates.sh

# Check MSRV of all the crates
check-crates-msrv:
	@bash contrib/scripts/check-crates.sh msrv

# Check Rust docs
check-docs:
	@bash contrib/scripts/check-docs.sh

# Release rust crates
[confirm]
release:
    @bash contrib/scripts/release.sh

# Run benches (unstable)
bench:
	RUSTFLAGS='--cfg=bench' cargo +nightly bench -p nostr

# Check cargo duplicate dependencies
dup:
    cargo tree -d

# Remove artifacts that cargo has generated
clean:
	cargo clean

# Build and serve the book
book:
    cd book && just serve

# Get many-events.json to test database performance
many-events:
	curl https://cdn.jb55.com/s/many-events.json.zst -o many-events.json.zst
	zstd -d many-events.json.zst

# Count the lines of codes of this project
loc:
	@echo "--- Counting lines of .rs files (LOC):" && find crates/ bindings/ -type f -name "*.rs" -not -path "*/target/*" -exec cat {} \; | wc -l
