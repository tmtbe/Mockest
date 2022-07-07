init:
	docker buildx create --name mybuilder --driver docker-container
	docker buildx use mybuilder
build:
	cd collector && docker buildx build --platform linux/arm,linux/arm64,linux/amd64 -t mockest/collector .
	cd demo && docker buildx build --platform linux/arm,linux/arm64,linux/amd64 -t mockest/demo .
	cd proxy && docker buildx build --platform linux/arm,linux/arm64,linux/amd64 -t mockest/proxy .

deploy.docker: build
	docker tag mockest/collector tmtbe/mockest-collector
	docker tag mockest/proxy tmtbe/mockest-proxy
	docker tag mockest/k8s-inject tmtbe/mockest-k8s-inject
	docker push tmtbe/mockest-proxy:latest
	docker push tmtbe/mockest-collector:latest
	docker push tmtbe/mockest-k8s-inject:latest

test.sandbox.record:
	docker network create mockest
	docker run -d --network mockest --name collector -v ${PWD}/replay:/home  mockest/collector
	docker run -d --cap-add=NET_ADMIN --cap-add=NET_RAW  --network mockest --name proxy mockest/proxy all
	docker run -d --network=container:proxy --name demo mockest/demo
	docker run -d --network mockest --name outbound-demo mockest/demo
test.sandbox.replay:
	docker network create mockest
	docker run -d -v ${PWD}/replay:/home/stubby4j/data --name replay --network mockest -e STUBS_PORT=80 azagniotov/stubby4j:latest-jre11
	docker run -d --network mockest --name collector  mockest/collector
	docker run -d --cap-add=NET_ADMIN --cap-add=NET_RAW  --network mockest --dns 127.0.0.1 --name proxy mockest/proxy all --replay
	docker run -d --network container:proxy --name coredns -v ${PWD}/coredns:/etc/coredns/ coredns/coredns -conf /etc/coredns/Corefile
	docker run -d --network=container:proxy --name demo mockest/demo
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