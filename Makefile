.PHONY: build.filter
build.filter:
	cd filter && nerdctl run -v "${PWD}":/home -w /home tinygo/tinygo:0.23.0 sh build.sh

.PHONY: build.sidecar
build.sidecar: build.filter
	cp ./filter/dist/filter.wasm ./docker/data/filter.wasm
	cd docker && nerdctl build -t test/sidecar .