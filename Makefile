precommit:
	@sh .githooks/pre-push

clean:
	cargo clean

rebase-bitcoin-v0.29:
	git checkout bitcoin-v0.29
	git rebase master
	git push origin --force-with-lease && git push upstream --force-with-lease
	git checkout master

loc:
	@echo "--- Counting lines of .rs files (LOC):" && find crates/ bindings/ -type f -name "*.rs" -exec cat {} \; | wc -l