# Use 'verbose=1' to echo all commands, for example 'make help verbose=1'.
ifdef verbose
  Q :=
else
  Q := @
endif

publish: pack
	$(Q)npm publish --access public

pack:
	$(Q)npm run package
