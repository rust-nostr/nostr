# Use 'verbose=1' to echo all commands, for example 'make help verbose=1'.
ifdef verbose
  Q :=
else
  Q := @
endif

precommit:
	$(Q)sh .githooks/pre-push

clean:
	$(Q)cargo clean

rebase-bitcoin-v0.29:
	$(Q)git checkout bitcoin-v0.29
	$(Q)git rebase master
	$(Q)git push origin --force-with-lease && git push upstream --force-with-lease
	$(Q)git checkout master

loc:
	$(Q)echo "--- Counting lines of .rs files (LOC):" && find crates/ bindings/ -type f -name "*.rs" -exec cat {} \; | wc -l