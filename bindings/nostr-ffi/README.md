# Nostr FFI

## Prerequisites

* `just`: https://just.systems/man/en/
* When building for Android:
  * Set the `ANDROID_SDK_ROOT` env variable
  * Set the `ANDROID_NDK_HOME` env variable

## Build

On first usage you will need to run:

```bash
just init
```

### Python

For most users, we recommend using our official Python package: [nostr-protocol](https://pypi.org/project/nostr-protocol/)

If you want to compile from source or need more options, read on.

### Wheel

```bash
just python
```

### Kotlin

For most users, we recommend using our official Kotlin package: [io.github.rust-nostr:nostr](https://central.sonatype.com/artifact/io.github.rust-nostr/nostr/).

If you want to compile from source or need more options, read on.

#### Libraries and Bindings

This command will build libraries for different platforms in `target/` folder and copy them to `ffi/kotlin/jniLibs`.
In addition it will generate Kotlin bindings in `ffi/kotlin/nostr`.

```bash
just kotlin
```

#### Android Archive (AAR)

This command will build an AAR file in `ffi/android/lib-release.aar`:

```bash
just bindings-android
```

See [Add your AAR or JAR as a dependency](https://developer.android.com/studio/projects/android-library#psd-add-aar-jar-dependency) in Android's docs for more information on how to integrate such an archive into your project.

### Swift

For most users, we recommend using our official Swift package: [rust-nostr/nostr-swift](https://github.com/rust-nostr/nostr-swift).

If you want to compile from source or need more options, read on.

#### Swift Module

These commands will build libraries for different architectures in `../../target/` and generate Swift bindings as well as Swift module artifacts in `ffi/swift-ios/` and `ffi/swift-darwin/` respectively:

```bash
just swift-ios
```

```bash
just swift-darwin
```

#### Swift Package

This command will produce a fully configured Swift Package in `bindings-swift/`.
See [Adding package dependencies to your app](https://developer.apple.com/documentation/xcode/adding-package-dependencies-to-your-app) in Apple's docs for more information on how to integrate such a package into your project.

```bash
just bindings-swift
```

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details