init:
	cargo install wasm-pack

publish: pack
	npm publish --access public

pack:
	rm -rf ./pkg
	npm run package
