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