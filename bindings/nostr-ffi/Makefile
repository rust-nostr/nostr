# Use 'verbose=1' to echo all commands, for example 'make help verbose=1'.
ifdef verbose
  Q :=
else
  Q := @
endif

all: build

build: deps
	$(Q)uniffi-bindgen generate src/nostr.udl --language kotlin --no-format --out-dir ./build/android/app/src/main/java
	$(Q)cargo ndk -t armeabi-v7a -t arm64-v8a -t x86_64 -t x86 -o build/android/app/src/main/jniLibs build --release

deps:
	$(Q)cargo install cargo-ndk --version 2.12.2
	$(Q)cargo install uniffi_bindgen --version 0.21.0
	$(Q)rustup target add \
	aarch64-linux-android \
	armv7-linux-androideabi \
	x86_64-linux-android \
	i686-linux-android

precommit:
	$(Q)cargo fmt && cargo clippy

clean:
	$(Q)cargo clean

loc:
	$(Q)echo "--- Counting lines of .rs files (LOC):" && find crates/ -type f -name "*.rs" -exec cat {} \; | wc -l