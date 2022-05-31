.PHONY: build.filter
build.filter:
	cd outbound_filter && cargo build --target wasm32-unknown-unknown --release

.PHONY: build.sidecar
build.sidecar: build.filter
	cp ./outbound_filter/target/wasm32-unknown-unknown/release/outbound_filter.wasm ./docker/data/outbound_filter.wasm
	cd docker && nerdctl build -t test/sidecar .