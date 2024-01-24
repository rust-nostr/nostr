fmt:
	cargo fmt --all -- --config format_code_in_doc_comments=true

check-fmt:
	cargo fmt --all -- --config format_code_in_doc_comments=true --check

check: fmt
	cargo build
	cargo clippy -- -D warnings