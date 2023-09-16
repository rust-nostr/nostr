precommit:
	@bash .githooks/pre-push

clean:
	cargo clean

loc:
	@echo "--- Counting lines of .rs files (LOC):" && find crates/ bindings/ -type f -name "*.rs" -exec cat {} \; | wc -l