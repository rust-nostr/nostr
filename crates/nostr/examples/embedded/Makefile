RUSTFLAGS := "-C link-arg=-Tlink.x"
CARGO_TARGET_THUMBV7M_NONE_EABI_RUNNER := "qemu-system-arm -cpu cortex-m3 -machine mps2-an385 -nographic -semihosting-config enable=on,target=native -kernel"

all: build

init:
	sudo apt install -y gcc-arm-none-eabi qemu-system-arm gdb-multiarch
	rustup +nightly target add thumbv7m-none-eabi

build:
	RUSTFLAGS=$(RUSTFLAGS) CARGO_TARGET_THUMBV7M_NONE_EABI_RUNNER=$(CARGO_TARGET_THUMBV7M_NONE_EABI_RUNNER) cargo +nightly build --release --target thumbv7m-none-eabi

run:
	RUSTFLAGS=$(RUSTFLAGS) CARGO_TARGET_THUMBV7M_NONE_EABI_RUNNER=$(CARGO_TARGET_THUMBV7M_NONE_EABI_RUNNER) cargo +nightly run --release --target thumbv7m-none-eabi