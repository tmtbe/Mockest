# Mockest
项目开展中。。
* 通过intercept拦截容器中的流量 【已实现】
* 通过collector收集拦截的流量日志 【已实现】
* 通过intercept强行进行串行请求，绑定inbound和outbound
* collector系列功能实现，UI
* 提供cli工具快速上手
* 通过replay实现重放


##
```
nerdctl run -it --cap-add=NET_ADMIN --cap-add=NET_RAW -v /Users/jincheng.zhang/envoy/:/home centeos:7 bash
```


```
FROM centos:7
RUN useradd -m --uid 1987 otdd-test-runner && \
    echo "otdd-test-runner ALL=NOPASSWD: ALL" >> /etc/sudoers
RUN yum install -y sudo iptables
```

```
sudo iptables -t nat -A OUTPUT -p tcp -m owner --uid-owner 1987 -j ACCEPT
sudo iptables -t nat -A OUTPUT -p tcp -j REDIRECT --to-port 18746
```

```shell
docker run -d --cap-add=NET_ADMIN --cap-add=NET_RAW  --name sidecar test/sidecar
docker run -d --net=container:sidecar ${test}
```