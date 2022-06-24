build.docker: build.proxy.docker build.collector.docker
clean:  clean.collector clean.proxy  clean.proxy.intercept clean.proxy.cmd

build.proxy.docker:clean.proxy build.proxy.intercept build.proxy.cmd
	cd proxy && cp -r docker target
	cp ./proxy/intercept/target/wasm32-unknown-unknown/release/intercept.wasm ./proxy/target/data/intercept.wasm
	cp ./proxy/cmd/target/proxy ./proxy/target/data/proxy
	cd ./proxy/target && docker build -t mockest/proxy .
clean.proxy:
	rm -rf ./proxy/target
build.proxy.intercept:
	cd proxy/intercept && cargo build --target wasm32-unknown-unknown --release
clean.proxy.intercept:
	rm -rf proxy/intercept/target
build.proxy.cmd:
	cd proxy/cmd && go mod tidy && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o ./target/proxy
clean.proxy.cmd:
	rm -rf proxy/cmd/target


build.collector:
	cd collector && go mod tidy && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o ./target/collector
build.collector.docker:clean.collector
	cd collector && cp -r docker target && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o ./target/collector
	cd ./collector/target && docker build -t mockest/collector .
clean.collector:
	rm -rf ./collector/target

build.k8s.inject:
	cd k8s/inject && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o ./target/inject
clean.k8s.inejct:
	rm -rf k8s/inject/target

deploy.docker: build.docker
	docker tag mockest/proxy tmtbe/mockest-proxy
	docker push tmtbe/mockest-proxy:latest

test.sandbox.record:
	docker network create mockest
	docker run -d --network mockest --name collector -v ${PWD}/replay:/home  mockest/collector
	docker run -d --cap-add=NET_ADMIN --cap-add=NET_RAW  --network mockest --name proxy mockest/proxy all
	docker run -d --network=container:proxy --name nginx nginx
test.sandbox.replay:
	docker network create mockest
	docker run -d --network mockest --name collector  mockest/collector
	docker run -d --cap-add=NET_ADMIN --cap-add=NET_RAW  --network mockest --dns 127.0.0.1 --name proxy mockest/proxy all --replay
	docker run -d --network container:proxy --name coredns -v ${PWD}/coredns:/etc/coredns/ coredns/coredns -conf /etc/coredns/Corefile
	docker run -d --network=container:proxy --name nginx nginx
	docker run -d -v ${PWD}/replay:/home/stubby4j/data --name replay --network mockest -e STUBS_PORT=80 azagniotov/stubby4j:latest-jre11
test.sandbox.clean:
	docker rm -f collector
	docker rm -f proxy
	docker rm -f nginx
	docker rm -f coredns
	docker rm -f replay
	docker network rm mockest
test.record: build.docker test.sandbox.clean test.sandbox.record
	docker run --network mockest alpine/curl curl proxy/test1
	docker run --network=container:proxy alpine/curl curl -k "https://hanyu.baidu.com/s?wd=%E4%B8%80&from=poem"
	docker run --network=container:proxy alpine/curl curl -k "https://www.bing.com/search?q=s&form=QBLH&sp=-1&pq=s&sc=8-1&qs=n&sk=&cvid=4B867E9C516F42FFAF3A9021D1ED9642"
	docker run --network mockest alpine/curl curl collector/gen
test.replay: build.docker test.sandbox.clean test.sandbox.replay
	sleep 5
	docker run --network mockest alpine/curl curl proxy/test1
	docker run --network=container:proxy alpine/curl curl -k "https://hanyu.baidu.com/s?wd=%E4%B8%80&from=poem"
	docker run --network=container:proxy alpine/curl curl -k "https://www.bing.com/search?q=s&form=QBLH&sp=-1&pq=s&sc=8-1&qs=n&sk=&cvid=4B867E9C516F42FFAF3A9021D1ED9642"
