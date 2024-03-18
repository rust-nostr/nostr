.PHONY: book precommit check

cli:##	cli
###    :
###make	cli
### 	additional cli help output
	cargo build -p nostr-cli --release

precommit: fmt check-crates check-bindings check-docs##	precommit
###    :
###make	precommit
### 	additional precommit help output

check: fmt check-crates check-crates-msrv check-bindings check-docs###	check
###    :
###make	checkout
### 	additional check help output

fmt:###	fmt
###    :
###make	fmt
### 	additional fmt help output
	@bash contrib/scripts/check-fmt.sh

check-fmt:###	check-fmt
###    :
###make	check-fmt
### 	additional check-fmt help output
	@bash contrib/scripts/check-fmt.sh check

check-bindings:###	check-bindings
###    :
###make	check-bindings
### 	additional check-bindings help output
	@bash contrib/scripts/check-bindings.sh

check-book:###	check-book
###    :
###make	check-book
### 	additional check-book help output
	@bash contrib/scripts/check-book.sh

check-crates:###	check-crates
###    :
###make	check-crates
### 	additional check-crates help output
	@bash contrib/scripts/check-crates.sh

check-crates-msrv:###	check-crates-msrv
###    :
###make	check-crates-msrv
### 	additional check-crates-msrv help output
	@bash contrib/scripts/check-crates.sh msrv

check-docs:###	check-docs
###    :
###make	check-docs
### 	additional check-docs help output
	@bash contrib/scripts/check-docs.sh

# Release rust crates
###    :
release:###	release
###make	release
### 	additional release help output
	@bash contrib/scripts/release.sh

bench:###	bench
###    :
###make	bench
### 	additional bench help output
	RUSTFLAGS='--cfg=bench' cargo +nightly bench -p nostr

clean:###	clean
###    :
###make	clean
### 	additional clean help output
	cargo clean
	cd bindings/nostr-js && cargo clean
	cd bindings/nostr-sdk-js && cargo clean

book:## 	book
###    :
###make	book
### 	additional book help output
	cd book && just serve || cargo install just

flatbuf:##	flatbuf
###    :
###make	flatbuf
###	additional flatbuf2 help
	cd crates/nostr-database && make flatbuf

loc:##	loc
###    :
###make	loc
### 	additional loc help output
	@echo "--- Counting lines of .rs files (LOC):" && find crates/ bindings/ -type f -name "*.rs" -not -path "*/target/*" -exec cat {} \; | wc -l

more:##
###    :
	@sed -n 's/^###//p' ${MAKEFILE_LIST} | column -t -s ':' |  sed -e 's/^/	/'
#	@sed -n 's/^### 	//p' ${MAKEFILE_LIST} | column -t -s ':' |  sed -e 's/^/	/'
#
#
# vim: set noexpandtab:
# vim: set setfiletype make
