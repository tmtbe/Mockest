# Mockest
Mockest无侵入的对被测应用的入口和出口流量拦截录制，实现被测应用外部依赖回放的能力。
* 在很多传统企业做遗留项目改造的过程中，缺乏相关测试，通过Mockest可以快速进行场景录制，将遗留项目的依赖录制下来进行回放，减轻手动编写大量Mock Server的工作。
* 在大型项目进行组件测试的过程中，同样也可以利用录制回放的能力减少手动编写Mock Server的工作。
* Mockest对录制对象无任何改造，只需按照步奏将被录制的对象运行在容器中即可。
* Mockest录制的结果为Stubby4j/4node支持的文件格式，通过一个Stubby4J实现所有外部依赖服务的Mock。

## 原理
* 通过envoy+iptables实现透明代理，所有的流量将被collector收集
* collector将收集到的所有流量进行整理生成用于回放的replay文件
* replay服务（Stubby）将读取replay文件进行mock
* coredns将强制解析外部的域名到0.0.0.0

## 如何使用
## 安装
```shell
    make build.docker
```
其中用nginx模拟被测应用
## record例子
```shell
	docker network create mockest
	docker run -d --network mockest --name collector -v ${PWD}/replay:/home  mockest/collector
	docker run -d --cap-add=NET_ADMIN --cap-add=NET_RAW  --network mockest --name sidecar mockest/sidecar
	docker run -d --network=container:sidecar --name nginx nginx
```
## replay例子
```shell
    docker network create mockest
	docker run -d --network mockest --name collector  mockest/collector
	docker run -d --cap-add=NET_ADMIN --cap-add=NET_RAW  --network mockest --dns 127.0.0.1 --name sidecar -e REPLAY=1 mockest/sidecar
	docker run -d --network container:sidecar --name coredns -v ${PWD}/coredns:/etc/coredns/ coredns/coredns -conf /etc/coredns/Corefile
	docker run -d --network=container:sidecar --name nginx nginx
	docker run -d -v ${PWD}/replay:/home/stubby4j/data --name replay --network mockest -e STUBS_PORT=80 azagniotov/stubby4j:latest-jre11
```