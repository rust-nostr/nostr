# Release Checks

* Run `just check` to verify that everything compile

* Try to compile `kotlin` bindings (`nostr-ffi` and `nostr-sdk-ffi`) since compilation could fail during gradlew due to enumerations names.

* Bump bindings libraries
  * Android in `gradle.properties`
  * Python in `setup.py`
  * Js in `package.json`
  * Swift Package not require version update

* Bump rust crates
    * Bump version in README.md files

* Commit and push (**without tag**)
    * `ffi: bump to vX.X.X`
    * `js: bump to vX.X.X`
    * `rust: bump to X.X.X`

* Release crates and bindings
    * Publish crates with `just release` or `bash ./contrib/scripts/release.sh`
    * Publish `kotlin` bindings
    * Publish `python` bindings
    * Publish `JS` bindings
    * Publish `Swift` bindings

* Bump versions in `book` (**without commit**, commit in next step)
    * Update examples
    * Rust book tests: `just check-book`
  
* Update `CHANGELOG.md`

* Commit and push (**WITH tag**)
    * `Release vX.X.X`
