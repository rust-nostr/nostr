# Release Checks

* Run `just check` to verify that everything compile

* Try to compile `kotlin` bindings (`nostr-sdk-ffi`) since compilation could fail during gradlew due to enumerations names.

* Bump bindings libraries
  * Android in `lib/build.gradle.kts`
  * Python in `setup.py`
  * Js in `package.json`
  * Flutter in `pubspec.yaml`
  * Swift Package NOT require version update

* Bump rust crates

* Commit and push (**without tag**): `Bump to vX.X.X`

* Release crates and bindings
    * Publish crates with `just release` or `bash ./contrib/scripts/release.sh`
    * Publish `kotlin` bindings
    * Publish `python` bindings
    * Publish `JS` bindings
    * Publish `Swift` bindings

* Bump versions in `book` (**without commit**, commit in next step)
    * Update examples
    * Search in the code for `UNCOMMENT_ON_RELEASE` string and uncomment the code (examples added in book before release)
    * Rust book tests: `just check-book`
  
* Update `CHANGELOG.md`

* Commit and push (**WITH tag**)
    * `Release vX.X.X`
