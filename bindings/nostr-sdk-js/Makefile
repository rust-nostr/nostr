.PHONY: check check-fmt fmt rustfmt

rustfmt:
	@rustup install nightly-2024-01-11
	@rustup component add rustfmt --toolchain nightly-2024-01-11

fmt: rustfmt
	cargo +nightly-2024-01-11 fmt --all -- --config format_code_in_doc_comments=true

check-fmt: rustfmt
	cargo fmt --all -- --config format_code_in_doc_comments=true --check

check: fmt
	cargo build
	cargo clippy -- -D warnings