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

# Run code coverage using cargo-llvm-cov.
#
# Requires:
# - cargo-llvm-cov (install via: cargo install cargo-llvm-cov)
# - llvm-tools-preview component (install via: rustup component add llvm-tools-preview)
coverage package='none':
    cargo llvm-cov clean --workspace
    cargo llvm-cov --html {{ if package == 'none' { '--workspace' } else { '--package ' + package } }}
    @echo
    @echo 'open {{ justfile_directory() }}/target/llvm-cov/html/index.html'

# Run benches
bench benchmark='none':
    RUSTFLAGS='--cfg=bench -Awarnings' cargo bench -p benches {{ if benchmark == 'none' { '' } else { '"'+benchmark+'"' } }}
    @echo
    @echo 'open {{ justfile_directory() }}/target/criterion/report/index.html'
