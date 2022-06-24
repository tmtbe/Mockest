# K8S 
## inject
提供inject命令修改deployment.yaml用于k8s录制环境的搭建。
```shell
inject -f hello.yaml
```
## collector部署
默认安装在default namespace下，如果被测应用在其他命名空间下需要自行调整
```shell
kubectl apply -f mockest/mockest.yaml
```