CURRENT_DIR := $(shell pwd)
SOURCES=$(sort $(wildcard ./src/*.rs ./src/**/*.rs))
UNAME := $(shell uname)

ifeq ($(UNAME), Linux)
COPY = cp
EXT = so
else ifeq ($(UNAME), Darwin)
COPY = cp
EXT = dylib
else
COPY = copy
EXT = dll
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

kotlin: android
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate src/nostr_sdk.udl --language kotlin --no-format -o ffi/kotlin

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
	cp -r ffi/kotlin/nostr_sdk bindings-android/lib/src/main/kotlin/
	cd bindings-android && ./gradlew assemble
	mkdir -p ffi/android
	cp bindings-android/lib/build/outputs/aar/lib-release.aar ffi/android

publish-bindings-android: bindings-android
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
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate src/nostr_sdk.udl -l swift -o ffi/swift-ios
	cp ../../target/ios-universal/release/libnostr_sdk_ffi.a ffi/swift-ios
	cd ffi/swift-ios && "swiftc" "-emit-module" "-module-name" "nostr_sdk_ffi"  "-Xcc" "-fmodule-map-file=$(CURRENT_DIR)/ffi/swift-ios/nostr_sdkFFI.modulemap" "-I" "."  "-L" "." "-lnostr_sdk_ffi" nostr_sdk.swift

swift-darwin: darwin-universal
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate src/nostr_sdk.udl -l swift -o ffi/swift-darwin
	cp ../../target/darwin-universal/release/libnostr_sdk_ffi.dylib ffi/swift-darwin
	cd ffi/swift-darwin && "swiftc" "-emit-module" "-module-name" "nostr_sdk_ffi"  "-Xcc" "-fmodule-map-file=$(CURRENT_DIR)/ffi/swift-darwin/nostr_sdkFFI.modulemap" "-I" "."  "-L" "." "-lnostr_sdk_ffi" nostr_sdk.swift

bindings-swift: ios-universal darwin-universal
	mkdir -p bindings-swift/Sources/NostrSDK
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate src/nostr_sdk.udl --no-format --language swift --out-dir bindings-swift/Sources/NostrSDK
	mv bindings-swift/Sources/NostrSDK/nostr_sdk.swift bindings-swift/Sources/NostrSDK/NostrSDK.swift
	cp bindings-swift/Sources/NostrSDK/nostr_sdkFFI.h bindings-swift/nostr_sdkFFI.xcframework/ios-arm64/nostr_sdkFFI.framework/Headers
	cp bindings-swift/Sources/NostrSDK/nostr_sdkFFI.h bindings-swift/nostr_sdkFFI.xcframework/ios-arm64_x86_64-simulator/nostr_sdkFFI.framework/Headers
	cp bindings-swift/Sources/NostrSDK/nostr_sdkFFI.h bindings-swift/nostr_sdkFFI.xcframework/macos-arm64_x86_64/nostr_sdkFFI.framework/Headers
	cp ../../target/aarch64-apple-ios/release/libnostr_sdk_ffi.a bindings-swift/nostr_sdkFFI.xcframework/ios-arm64/nostr_sdkFFI.framework/nostr_sdkFFI
	cp ../../target/ios-universal-sim/release/libnostr_sdk_ffi.a bindings-swift/nostr_sdkFFI.xcframework/ios-arm64_x86_64-simulator/nostr_sdkFFI.framework/nostr_sdkFFI
	cp ../../target/darwin-universal/release/libnostr_sdk_ffi.a bindings-swift/nostr_sdkFFI.xcframework/macos-arm64_x86_64/nostr_sdkFFI.framework/nostr_sdkFFI
	rm bindings-swift/Sources/NostrSDK/nostr_sdkFFI.h
	rm bindings-swift/Sources/NostrSDK/nostr_sdkFFI.modulemap

python-dev:
	pip install -r bindings-python/requirements.txt
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate src/nostr_sdk.udl --language python --no-format -o bindings-python/src/nostr-sdk/
	cargo build --release
	$(COPY) ../../target/release/libnostr_sdk_ffi.$(EXT) bindings-python/src/nostr-sdk/
	cd bindings-python && python setup.py --verbose bdist_wheel
	pip install ./bindings-python/dist/nostr_sdk-*.whl --force-reinstall
