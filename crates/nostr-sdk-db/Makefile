all: build

flatbuf:
	flatc --rust -o ./src/flatbuffers ./fbs/event.fbs

graph:
	CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --release --example indexes -o flamegraph.svg