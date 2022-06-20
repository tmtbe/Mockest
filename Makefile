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

test.sandbox.record:
	docker network create mockest
	docker run -d --network mockest --name collector  mockest/collector
	docker run -d --cap-add=NET_ADMIN --cap-add=NET_RAW  --network mockest --name sidecar mockest/sidecar
	docker run -d --network=container:sidecar --name nginx nginx
test.sandbox.replay:
	docker network create mockest
	docker run -d --network mockest --name collector  mockest/collector
	docker run -d --cap-add=NET_ADMIN --cap-add=NET_RAW  --network mockest --dns 127.0.0.1 --name sidecar -e REPLAY=1 mockest/sidecar
	docker run -d --network container:sidecar --name coredns -v ${PWD}/coredns:/etc/coredns/ coredns/coredns -conf /etc/coredns/Corefile
test.sandbox.clean:
	docker rm -f `docker ps -qa`
	docker network rm mockest
test.record: build.docker test.sandbox.clean test.sandbox.record
	docker run --network mockest centos:7 curl sidecar
	docker run --network=container:sidecar centos:7 curl "http://www.baidu.com"
	docker run --network=container:sidecar centos:7 curl -k "https://www.baidu.com"
test.replay: build.docker test.sandbox.clean test.sandbox.replay
	docker run --network=container:sidecar centos:7 curl -k "https://www.baidu.com"
