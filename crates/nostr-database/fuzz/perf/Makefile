graph:
	RUST_LOG=info CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --release -o flamegraph.svg

many-events.json:
	curl https://cdn.jb55.com/s/many-events.json.zst -o many-events.json.zst
	zstd -d many-events.json.zst
