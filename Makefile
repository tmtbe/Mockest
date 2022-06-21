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
	docker run -d --network mockest --name collector -v ${PWD}/replay:/home  mockest/collector
	docker run -d --cap-add=NET_ADMIN --cap-add=NET_RAW  --network mockest --name sidecar mockest/sidecar
	docker run -d --network=container:sidecar --name nginx nginx
test.sandbox.replay:
	docker network create mockest
	docker run -d --network mockest --name collector  mockest/collector
	docker run -d --cap-add=NET_ADMIN --cap-add=NET_RAW  --network mockest --dns 127.0.0.1 --name sidecar -e REPLAY=1 mockest/sidecar
	docker run -d --network container:sidecar --name coredns -v ${PWD}/coredns:/etc/coredns/ coredns/coredns -conf /etc/coredns/Corefile
	docker run -d --network=container:sidecar --name nginx nginx
	docker run -d -v ${PWD}/replay:/home/stubby4j/data --name replay --network mockest -e STUBS_PORT=80 azagniotov/stubby4j:latest-jre11
test.sandbox.clean:
	docker rm -f `docker ps -qa`
	docker network rm mockest
test.record: build.docker test.sandbox.clean test.sandbox.record
	docker run --network mockest alpine/curl curl sidecar/test1
	docker run --network=container:sidecar alpine/curl curl -k "https://hanyu.baidu.com/s?wd=%E4%B8%80&from=poem"
	docker run --network=container:sidecar alpine/curl curl -k "https://www.bing.com/search?q=s&form=QBLH&sp=-1&pq=s&sc=8-1&qs=n&sk=&cvid=4B867E9C516F42FFAF3A9021D1ED9642"
	docker run --network mockest alpine/curl curl collector/gen
test.replay: build.docker test.sandbox.clean test.sandbox.replay
	sleep 5
	docker run --network mockest alpine/curl curl sidecar/test1
	docker run --network=container:sidecar alpine/curl curl -k "https://hanyu.baidu.com/s?wd=%E4%B8%80&from=poem"
	docker run --network=container:sidecar alpine/curl curl -k "https://www.bing.com/search?q=s&form=QBLH&sp=-1&pq=s&sc=8-1&qs=n&sk=&cvid=4B867E9C516F42FFAF3A9021D1ED9642"
