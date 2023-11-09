all: build

flatbuf:
	flatc --rust -o ./src/flatbuffers ./fbs/event.fbs
	flatc --rust -o ./src/flatbuffers ./fbs/event_seen_by.fbs

graph:
	CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --release --example indexes -o flamegraph.svg