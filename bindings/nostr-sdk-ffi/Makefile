CURRENT_DIR := $(shell pwd)
SOURCES=$(sort $(wildcard ./src/*.rs ./src/**/*.rs))

ifeq ($(OS),Windows_NT)
    # Windows
    LIB_UNIFFI_EXT := dll
else
    UNAME_S := $(shell uname -s)
    ifeq ($(UNAME_S),Linux)
        # Linux
        LIB_UNIFFI_EXT := so

    endif
    ifeq ($(UNAME_S),Darwin)
        # macOS
        LIB_UNIFFI_EXT := a
    endif
endif

.PHONY: init
init:
	rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim
	rustup target add aarch64-apple-darwin x86_64-apple-darwin
	rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
	@if [ $$(uname) == "Darwin" ] ; then cargo install cargo-lipo ; fi
	cargo install cbindgen
	cargo install cargo-ndk

.PHONY: ndk-home
ndk-home:
	@if [ ! -d "${ANDROID_NDK_HOME}" ] ; then \
		echo "Error: Please, set the ANDROID_NDK_HOME env variable to point to your NDK folder" ; \
		exit 1 ; \
	fi

.PHONY: sdk-root
sdk-root:
	@if [ ! -d "${ANDROID_SDK_ROOT}" ] ; then \
		echo "Error: Please, set the ANDROID_SDK_ROOT env variable to point to your SDK folder" ; \
		exit 1 ; \
	fi

clean-android:
	rm -rf ffi/android
	rm -rf ffi/kotlin

kotlin: clean-android android
	find ./ffi/kotlin/jniLibs -name libnostr_ffi.so -type f -delete
	cargo run -p uniffi-bindgen generate --library ../../target/x86_64-linux-android/release/libnostr_sdk_ffi.so --language kotlin --no-format -o ffi/kotlin

android: aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

aarch64-linux-android: $(SOURCES) ndk-home
	cargo ndk -t aarch64-linux-android -o ffi/kotlin/jniLibs build --release	

armv7-linux-androideabi: $(SOURCES) ndk-home
	cargo ndk -t armv7-linux-androideabi -o ffi/kotlin/jniLibs build --release	

i686-linux-android: $(SOURCES) ndk-home
	cargo ndk -t i686-linux-android -o ffi/kotlin/jniLibs build --release	

x86_64-linux-android: $(SOURCES) ndk-home
	cargo ndk -t x86_64-linux-android -o ffi/kotlin/jniLibs build --release

bindings-android: sdk-root kotlin
	rm -rf bindings-android/lib/src/main/jniLibs
	rm -rf bindings-android/lib/src/main/kotlin
	cp -r ffi/kotlin/jniLibs bindings-android/lib/src/main
	mkdir -p bindings-android/lib/src/main/kotlin/
	cp -r ffi/kotlin/rust bindings-android/lib/src/main/kotlin/
	cd bindings-android && ./gradlew assembleRelease
	mkdir -p ffi/android
	cp bindings-android/lib/build/outputs/aar/lib-release.aar ffi/android

publish-android: bindings-android
	cd bindings-android && ./gradlew publishToSonatype closeAndReleaseSonatypeStagingRepository

ios-universal: $(SOURCES)		
	mkdir -p ../../target/ios-universal/release
	mkdir -p ../../target/ios-universal-sim/release
	cargo build --release --target aarch64-apple-ios
	cargo build --release --target x86_64-apple-ios
	cargo build --release --target aarch64-apple-ios-sim
	lipo -create -output ../../target/ios-universal/release/libnostr_sdk_ffi.a ../../target/aarch64-apple-ios/release/libnostr_sdk_ffi.a ../../target/x86_64-apple-ios/release/libnostr_sdk_ffi.a
	lipo -create -output ../../target/ios-universal-sim/release/libnostr_sdk_ffi.a ../../target/aarch64-apple-ios-sim/release/libnostr_sdk_ffi.a ../../target/x86_64-apple-ios/release/libnostr_sdk_ffi.a

darwin-universal: $(SOURCES)
	mkdir -p ../../target/darwin-universal/release
	cargo lipo --release --targets aarch64-apple-darwin
	cargo lipo --release --targets x86_64-apple-darwin
	lipo -create -output ../../target/darwin-universal/release/libnostr_sdk_ffi.dylib ../../target/aarch64-apple-darwin/release/libnostr_sdk_ffi.dylib ../../target/x86_64-apple-darwin/release/libnostr_sdk_ffi.dylib
	lipo -create -output ../../target/darwin-universal/release/libnostr_sdk_ffi.a ../../target/aarch64-apple-darwin/release/libnostr_sdk_ffi.a ../../target/x86_64-apple-darwin/release/libnostr_sdk_ffi.a

swift-ios: ios-universal
	cargo run -p uniffi-bindgen generate --library ../../target/aarch64-apple-ios/release/libnostr_sdk_ffi.a -l swift -o ffi/swift-ios
	cp ../../target/ios-universal/release/libnostr_sdk_ffi.a ffi/swift-ios
	cd ffi/swift-ios && "swiftc" "-emit-module" "-module-name" "nostr_sdk_ffi"  "-Xcc" "-fmodule-map-file=$(CURRENT_DIR)/ffi/swift-ios/nostr_sdkFFI.modulemap" "-I" "."  "-L" "." "-lnostr_sdk_ffi" nostr_sdk.swift

swift-darwin: darwin-universal
	cargo run -p uniffi-bindgen generate --library ../../target/aarch64-apple-darwin/release/libnostr_sdk_ffi.a -l swift -o ffi/swift-darwin
	cp ../../target/darwin-universal/release/libnostr_sdk_ffi.dylib ffi/swift-darwin
	cd ffi/swift-darwin && "swiftc" "-emit-module" "-module-name" "nostr_sdk_ffi"  "-Xcc" "-fmodule-map-file=$(CURRENT_DIR)/ffi/swift-darwin/nostr_sdkFFI.modulemap" "-I" "."  "-L" "." "-lnostr_sdk_ffi" nostr_sdk.swift

bindings-swift: ios-universal darwin-universal
	mkdir -p bindings-swift/Sources/NostrSDK
	cargo run -p uniffi-bindgen generate --library ../../target/aarch64-apple-ios/release/libnostr_sdk_ffi.a --no-format --language swift --out-dir bindings-swift/Sources/NostrSDK
	mv bindings-swift/Sources/NostrSDK/nostr.swift bindings-swift/Sources/NostrSDK/Nostr.swift
	mv bindings-swift/Sources/NostrSDK/nostr_sdk.swift bindings-swift/Sources/NostrSDK/NostrSDK.swift
	cp bindings-swift/Sources/NostrSDK/*.h bindings-swift/nostr_sdkFFI.xcframework/ios-arm64/Headers
	cp bindings-swift/Sources/NostrSDK/*.h bindings-swift/nostr_sdkFFI.xcframework/ios-arm64_x86_64-simulator/Headers
	cp bindings-swift/Sources/NostrSDK/*.h bindings-swift/nostr_sdkFFI.xcframework/macos-arm64_x86_64/Headers
	cp ../../target/aarch64-apple-ios/release/libnostr_sdk_ffi.a bindings-swift/nostr_sdkFFI.xcframework/ios-arm64/nostr_sdkFFI.a
	cp ../../target/ios-universal-sim/release/libnostr_sdk_ffi.a bindings-swift/nostr_sdkFFI.xcframework/ios-arm64_x86_64-simulator/nostr_sdkFFI.a
	cp ../../target/darwin-universal/release/libnostr_sdk_ffi.a bindings-swift/nostr_sdkFFI.xcframework/macos-arm64_x86_64/nostr_sdkFFI.a
	rm bindings-swift/Sources/NostrSDK/*.h
	rm bindings-swift/Sources/NostrSDK/*.modulemap

python:
	rm -rf bindings-python/dist
	pip install -r bindings-python/requirements.txt --break-system-packages
	cargo build --release
	cargo run -p uniffi-bindgen generate --library ../../target/release/libnostr_sdk_ffi.$(LIB_UNIFFI_EXT) --language python --no-format -o bindings-python/src/nostr-sdk/
	cp ../../target/release/libnostr_sdk_ffi.so bindings-python/src/nostr-sdk/ | true
	cp ../../target/release/libnostr_sdk_ffi.dylib bindings-python/src/nostr-sdk/ | true
	cd bindings-python && python setup.py --verbose bdist_wheel
	pip install ./bindings-python/dist/nostr_sdk-*.whl --force-reinstall --break-system-packages

python-win:
	pip install -r bindings-python\requirements.txt
	cargo build --release
	cargo run -p uniffi-bindgen generate --library ..\..\target\release\nostr_sdk_ffi.dll --language python --no-format -o bindings-python\src\nostr-sdk
	copy ..\..\target\release\nostr_sdk_ffi.dll bindings-python\src\nostr-sdk
	del /F /Q bindings-python\dist\* 2>nul || exit /b 0
	cd bindings-python && python setup.py --verbose bdist_wheel
	FOR %%i in (.\bindings-python\dist\*.whl) DO pip install %i --force-reinstall
