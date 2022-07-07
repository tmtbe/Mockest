init:
	docker buildx create --name mybuilder --driver docker-container
	docker buildx use mybuilder
build:
	cd collector && docker buildx build --platform linux/arm64,linux/amd64 -t tmtbe/mockest-collector:master .
	cd demo && docker buildx build --platform linux/arm64,linux/amd64 -t tmtbe/mockest-demo:master .
	cd proxy && docker buildx build --platform linux/arm64,linux/amd64 -t tmtbe/mockest-proxy:master .

build.easy:
	cd collector && docker build -t mockest/collector:master .
	cd demo && docker build -t mockest/demo:master .
	cd proxy && docker build -t mockest/proxy:master .

test.sandbox.record:
	docker network create mockest
	docker run -d --network mockest --name collector -v ${PWD}/replay:/home  tmtbe/mockest-collector:master
	docker run -d --cap-add=NET_ADMIN --cap-add=NET_RAW  --network mockest --name proxy tmtbe/mockest-proxy:master all
	docker run -d --network=container:proxy --name demo tmtbe/mockest-demo:master
	docker run -d --network mockest --name outbound-demo tmtbe/mockest-demo:master
test.sandbox.replay:
	docker network create mockest
	docker run -d -v ${PWD}/replay:/home/stubby4j/data --name replay --network mockest -e STUBS_PORT=80 azagniotov/stubby4j:latest-jre11
	docker run -d --network mockest --name collector  tmtbe/mockest-collector:master
	docker run -d --cap-add=NET_ADMIN --cap-add=NET_RAW  --network mockest --dns 127.0.0.1 --name proxy tmtbe/mockest-proxy:master all --replay
	docker run -d --network container:proxy --name coredns -v ${PWD}/coredns:/etc/coredns/ coredns/coredns -conf /etc/coredns/Corefile
	docker run -d --network=container:proxy --name demo tmtbe/mockest-demo:master
test.sandbox.clean:
	docker rm -f collector
	docker rm -f proxy
	docker rm -f demo
	docker rm -f coredns
	docker rm -f replay
	docker network rm mockest
test.record: test.sandbox.clean test.sandbox.record
	docker run --network mockest alpine/curl curl proxy/inbound
	docker run --network mockest alpine/curl curl collector/gen
test.replay: test.sandbox.clean test.sandbox.replay
	sleep 5
	docker run --network mockest alpine/curl curl proxy/inbound
	docker run --network mockest alpine/curl curl proxy/inbound