build.docker: build.record-sidecar.docker build.replay-sidecar.docker build.collector.docker
clean:  clean.collector clean.record-sidecar clean.replay-sidecar clean.record-sidecar.intercept clean.replay-sidecar.intercept

build.record-sidecar.docker:clean.record-sidecar build.record-sidecar.intercept
	cd record-sidecar && cp -r docker target
	cp ./envoy/envoy ./record-sidecar/target/data/envoy
	cp ./record-sidecar/intercept/target/wasm32-unknown-unknown/release/intercept.wasm ./record-sidecar/target/data/intercept.wasm
	cd ./record-sidecar/target && nerdctl build -t mockest/record-sidecar .
clean.record-sidecar:
	rm -rf ./record-sidecar/target
build.record-sidecar.intercept:
	cd record-sidecar/intercept && cargo build --target wasm32-unknown-unknown --release
clean.record-sidecar.intercept:
	rm -rf record-sidecar/intercept/target

build.replay-sidecar.docker:clean.replay-sidecar build.replay-sidecar.intercept
	cd replay-sidecar && cp -r docker target
	cp ./envoy/envoy ./replay-sidecar/target/data/envoy
	cp ./replay-sidecar/intercept/target/wasm32-unknown-unknown/release/intercept.wasm ./replay-sidecar/target/data/intercept.wasm
	cd ./replay-sidecar/target && nerdctl build -t mockest/replay-sidecar .
clean.replay-sidecar:
	rm -rf ./replay-sidecar/target
build.replay-sidecar.intercept:
	cd replay-sidecar/intercept && cargo build --target wasm32-unknown-unknown --release
clean.replay-sidecar.intercept:
	rm -rf replay-sidecar/intercept/target

build.collector:
	cd collector && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o ./target/collector
build.collector.docker:clean.collector
	cd collector && cp -r docker target && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o ./target/collector
	cd ./collector/target && nerdctl build -t mockest/collector .
clean.collector:
	rm -rf ./collector/target

test.sandbox:
	nerdctl network create mockest
	nerdctl run -d --cap-add=NET_ADMIN --cap-add=NET_RAW  --network mockest --name intercept  mockest/intercept
	nerdctl run -d --network mockest --name collector  mockest/collector
test.sandbox.clean:
	nerdctl network rm mockest
	nerdctl rm -f `nerdctl ps -qa`