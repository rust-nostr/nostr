.PHONY: book

precommit:
	@bash .githooks/pre-push

clean:
	cargo clean

book:
	cd book && make build

loc:
	@echo "--- Counting lines of .rs files (LOC):" && find crates/ bindings/ -type f -name "*.rs" -exec cat {} \; | wc -l