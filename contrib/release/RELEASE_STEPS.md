# Release Checks

* Run `just check` to verify that everything compile

* Try to compile `kotlin` bindings (`nostr-sdk-ffi`) since compilation could fail during gradlew due to enumerations names.

* Bump versions
  * Rust in various `Cargo.toml`
  * Android in `lib/build.gradle.kts`
  * Python in `setup.py`
  * Flutter in `pubspec.yaml` (other repository)
  * JavaScript in `package.json`
  * Swift Package NOT require version update

* Commit and push (**without tag**): `Bump to vX.X.X`

* Release crates and bindings
    * Publish crates with `just release` or `bash ./contrib/scripts/release.sh`
    * Publish `Kotlin` bindings
    * Publish `Python` bindings
    * Publish `JavaScript` bindings
    * Publish `Swift` bindings
    * Publish `Flutter` bindings (other repository)

* Bump versions in `book` (**without commit**, commit in next step)
    * Update examples
    * Search in the code for `UNCOMMENT_ON_RELEASE` string and uncomment the code (examples added in book before release)
    * Rust book tests: `just check-book`
  
* Update `CHANGELOG.md`

* Commit and push (**WITH tag**)
    * `Release vX.X.X`
