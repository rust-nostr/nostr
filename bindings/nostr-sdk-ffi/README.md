# Nostr SDK FFI

## Prerequisites

* When building for Android:
  * Set the `ANDROID_SDK_ROOT` env variable
  * Set the `ANDROID_NDK_HOME` env variable

## Build

On first usage you will need to run:

```
make init
```

### Python

For most users, we recommend using our official Python package: [nostr-sdk](https://pypi.org/project/nostr-sdk/).

If you want to compile from source or need more options, read on.

### Wheel

```
make python-dev
```

### Kotlin

For most users, we recommend using our official Kotlin package: [io.github.rust-nostr:nostr-sdk](https://central.sonatype.com/artifact/io.github.rust-nostr/nostr-sdk/).

If you want to compile from source or need more options, read on.

#### Libraries and Bindings

This command will build libraries for different platforms in `target/` folder and copy them to `ffi/kotlin/jniLibs`.
In addition it will generate Kotlin bindings in `ffi/kotlin/nostr_sdk`.

```
make kotlin
```

#### Android Archive (AAR)

This command will build an AAR file in `ffi/android/lib-release.aar`:

```
make bindings-android
```

See [Add your AAR or JAR as a dependency](https://developer.android.com/studio/projects/android-library#psd-add-aar-jar-dependency) in Android's docs for more information on how to integrate such an archive into your project.

### Swift

For most users, we recommend using our official Swift package: [rust-nostr/nostr-sdk-swift](https://github.com/rust-nostr/nostr-sdk-swift).

If you want to compile from source or need more options, read on.

#### Swift Module

These commands will build libraries for different architectures in `../../target/` and generate Swift bindings as well as Swift module artifacts in `ffi/swift-ios/` and `ffi/swift-darwin/` respectively:

```
make swift-ios
```

```
make swift-darwin
```

#### Swift Package

This command will produce a fully configured Swift Package in `bindings-swift/`.
See [Adding package dependencies to your app](https://developer.apple.com/documentation/xcode/adding-package-dependencies-to-your-app) in Apple's docs for more information on how to integrate such a package into your project.

```
make bindings-swift
```

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details