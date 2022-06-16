build.docker: build.sidecar.docker build.collector.docker
clean:  clean.collector clean.sidecar  clean.sidecar.intercept

build.sidecar.docker:clean.sidecar build.sidecar.intercept
	cd sidecar && cp -r docker target
	cp ./sidecar/intercept/target/wasm32-unknown-unknown/release/intercept.wasm ./sidecar/target/data/intercept.wasm
	cd ./sidecar/target && docker build -t mockest/sidecar .
clean.sidecar:
	rm -rf ./sidecar/target
build.sidecar.intercept:
	cd sidecar/intercept && cargo build --target wasm32-unknown-unknown --release
clean.sidecar.intercept:
	rm -rf sidecar/intercept/target


build.collector:
	cd collector && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o ./target/collector
build.collector.docker:clean.collector
	cd collector && cp -r docker target && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o ./target/collector
	cd ./collector/target && docker build -t mockest/collector .
clean.collector:
	rm -rf ./collector/target

test.sandbox:
	docker network create mockest
	docker run -d --cap-add=NET_ADMIN --cap-add=NET_RAW  --network mockest --name intercept  mockest/sidecar
	docker run -d --network mockest --name collector  mockest/collector
test.sandbox.clean:
	docker rm -f `docker ps -qa`
	docker network rm mockest
