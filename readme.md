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
## 镜像
* tmtbe/mockest-proxy:latest
* tmtbe/mockest-k8s-inject:latest
### 安装
```shell
    make build.docker
```
### record例子
```shell
	make test.record
```
### replay例子
```shell
   make test.replay
```

## 注意事项
由于HTTP/1.1协议的TLS连接可能没有采用ALPN协议，envoy无法识别加密流量到底是不是HTTP协议。我们默认将443端口识别成为HTTPS协议，但其他端口如果采用了非ALPN模式的HTTPS请求可能会导致无法识别。

解决方案：
* 采用ALPN方式请求
* 增加端口匹配或者域名匹配等等