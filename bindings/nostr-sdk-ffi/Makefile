# Use 'verbose=1' to echo all commands, for example 'make help verbose=1'.
ifdef verbose
  Q :=
else
  Q := @
endif

uniffideps:
	$(Q)cargo install uniffi_bindgen --version 0.23

publish-python: uniffideps python-deps
	$(Q)maturin publish

python-deps:
	$(Q)cargo install --locked maturin
