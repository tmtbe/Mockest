.PHONY: build.filter
build.filter:
	cd filter && cargo build --target wasm32-unknown-unknown --release

.PHONY: build.sidecar
build.sidecar: build.filter
	cp ./filter/target/wasm32-unknown-unknown/release/filter.wasm ./docker/data/filter.wasm
	cp ./filter/src/envoy.yaml ./docker/data/envoy.yaml
	cd docker && nerdctl build -t test/sidecar .