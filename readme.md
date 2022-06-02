# Mockest
项目开展中。。也招募志同道合的小伙伴们共同参与。

## 项目背景
* 在很多传统企业做遗留项目改造的过程中
* 在大型项目进行组件测试的过程中
你是否因为准备依赖测试的mock server而头疼，你是否因为要为遗留项目补充测试用例而头疼。
Mockest通过拦截被测应用的请求可以实现无侵入的数据录制功能，通过回放录制数据达到mock server的效果。

## 原理
类似istio的流量拦截，我们也是通过envoy拦截容器inbound，outbound的流量。
```
录制过程
inbound[intercept] <-> sut <-> outbound[intercept] <-> services
            push -> [collector]             push -> [collector] 
                    record inbound                  record outbound
回放过程
inbound[intercept] <-> sut <-> outbound[intercept] 
            push -> [replayer]           redirect -> [replayer] 
                    prepare replay data               replay data                                
```
inbound intercept 将强行将请求串行化，这样才能识别inbound和outbound的关系。

## 特性
* intercept系列功能：拦截容器中的流量 【已实现】
* intercept系列功能：collector功能支持 【已实现】
* intercept系列功能：inbound串行化访问，绑定inbound和outbound关系
* intercept系列功能：replayer功能支持
* collector系列功能：分类记录record数据
* collector系列功能：访问或下载原始录制数据
* collector系列功能：设置数据匹配规则
* collector系列功能：生成replayer使用的record数据
* collector系列功能：生成其他mockserver使用的数据
* collector系列功能：UI管理
* replayer系列功能：读取record数据，获取匹配规则
* replayer系列功能：通过request数据匹配response数据
* replayer系列功能：动态设置mock数据
* 提供cli工具快速上手

由于全局tcp拦截的存在，我们可以实现很多动态mock的功能，不必修改代码中的host，不必分host:port提供mock server。

