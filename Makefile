build.docker: build.intercept.docker build.collector.docker

build.intercept:
	cd intercept && cargo build --target wasm32-unknown-unknown --release
build.intercept.docker:build.intercept
	cp ./intercept/target/wasm32-unknown-unknown/release/intercept.wasm ./intercept/docker/data/intercept.wasm
	cp ./intercept/envoy.yaml ./intercept/docker/data/envoy.yaml
	cd ./intercept/docker && nerdctl build -t mockest/intercept .

build.collector:
	cd collector && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o ./target/collector
build.collector.docker:build.collector
	cp ./collector/target/collector ./collector/docker
	cd ./collector/docker && nerdctl build -t mockest/collector .