#!/usr/bin/env just --justfile

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

[private]
default:
    @just --list

# Execute the pre-commit checks
precommit: fmt check-crates check-docs

# Execute continuous integration (CI) checks
ci: check-fmt check-crates check-docs

# Format the entire Rust code
fmt:
	@bash contrib/scripts/fmt.sh

# Check if the Rust code is formatted
[private]
check-fmt:
	@bash contrib/scripts/fmt.sh check

# Check all the crates
[private]
check-crates:
	@bash contrib/scripts/check-crates.sh

# Check Rust docs
[private]
check-docs:
	@bash contrib/scripts/check-docs.sh

# Release rust crates
[confirm]
release:
    cargo +stable publish --workspace

# Run benches (unstable)
bench:
	RUSTFLAGS='--cfg=bench' cargo +nightly bench
