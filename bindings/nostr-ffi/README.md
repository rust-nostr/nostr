# Nostr FFI

## Prerequisites
* When building for Android:
  * Set the `ANDROID_SDK_ROOT` env variable
  * Set the `ANDROID_NDK_HOME` env variable

## Build

On first usage you will need to run:

```
make init
```

### Kotlin

### Libraries and Bindings

This command will build libraries for different platforms in `target/` folder and copy them to `ffi/kotlin/jniLibs`.
In addition it will generate Kotlin bindings in `ffi/kotlin/nostr`.

```
make kotlin
```

### Android Archive (AAR)

This command will build an AAR file in `ffi/android/lib-release.aar`:

```
make bindings-android
```

See [Add your AAR or JAR as a dependency](https://developer.android.com/studio/projects/android-library#psd-add-aar-jar-dependency) in Android's docs for more information on how to integrate such an archive into your project.

## License

This project is distributed under the MIT software license - see the [LICENSE](../../LICENSE) file for details