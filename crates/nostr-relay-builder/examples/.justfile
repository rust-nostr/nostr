default:
    @just --choose

hyper:
    @RUST_LOG=debug \
    cargo run --example hyper

local-with-hs:
    @RUST_LOG=debug,tor_rtcompat=trace,arti_client=trace \
    cargo run --example local-with-hs --features tor

mock:
    @RUST_LOG=debug \
    cargo run --example mock

policy:
    @RUST_LOG=debug \
    cargo run --example policy


