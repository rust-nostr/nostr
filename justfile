#!/usr/bin/env just --justfile

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

[private]
default:
    @just --list

# Execute the pre-commit checks
precommit: fmt check-crates check-docs

# Format the entire Rust code
fmt:
	@bash contrib/scripts/check-fmt.sh

# Check if the Rust code is formatted
check-fmt:
	@bash contrib/scripts/check-fmt.sh check

# Check all the crates
check-crates:
	@bash contrib/scripts/check-crates.sh

# Check Rust docs
check-docs:
	@bash contrib/scripts/check-docs.sh

# Check cargo-deny
check-deny:
	@bash contrib/scripts/check-deny.sh

# Release rust crates
[confirm]
release:
    cargo +stable publish --workspace

# Run benches (unstable)
bench:
	RUSTFLAGS='--cfg=bench' cargo +nightly bench

# Count the lines of codes of this project
loc:
	@echo "--- Counting lines of .rs files (LOC):" && find -type f -name "*.rs" -not -path "*/target/*" -exec cat {} \; | wc -l
