.PHONY: build.filter
build.filter:
	cd filter && cargo build --target wasm32-unknown-unknown --release

.PHONY: build.sidecar
build.sidecar: build.filter
	cp ./filter/dist/filter.wasm ./docker/data/filter.wasm
	cd docker && nerdctl build -t test/sidecar .