# Release Checks

* Run `make check` to verify that eveything compile

* Try to compile `kotlin` bindings (`nostr-ffi` and `nostr-sdk-ffi`) since compilation could fail during gradlew due to enumerations names.

* Bump bindings libraries

* Bump rust crates
    * Bump version in README.md files

* Commit and push (**without tag**)
    * `ffi: bump to vX.X.X`
    * `js: bump to vX.X.X`
    * `rust: bump to X.X.X`

* Release crates and bindings
    * Publish crates with `make release` or `bash ./contrib/scripts/release.sh`
    * Publish `kotlin` bindings
    * Publish `python` bindings
    * Publish `JS` bindings
    * Publish `Swift` bindings

* Bump versions in `book`
    * Update examples
    * Rust book tests: `make check-book`

* Commit and push (**WITH tag**)
    * `Release vX.X.X`
