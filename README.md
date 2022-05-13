## 介绍
本项目主要为解决研究及学习人员访问`Github`过慢或其他问题而提供的免费的`Github hosts`同步服务。

本项目部分参考 [521xueweihan/Github520](https://github.com/521xueweihan/GitHub520)，但与之不同的是前者是通过`ipaddress.com`获取`github.com`的`hosts`，而本项目是通过部署此项目的服务器来获取`github.com`的`hosts`，所以在IP节点上会存在一定的差异。

## 使用方法
### Unix/Linux
```shell
sed -i "/# fetch-github-hosts begin/Q" /etc/hosts && curl https://hosts.gitcdn.top/hosts.txt >> /etc/hosts
```
> 提示：可以设置定时任务定时获取更新即可

### Chrome
使用 [FasterHosts](https://github.com/gauseen/faster-hosts) 插件

### Windows/MacOS及其他桌面端
使用 [https://github.com/oldj/SwitchHosts](SwitchHosts) 程序，添加远程规则即可，远程地址为：https://hosts.gitcdn.top/hosts.txt

### 开源协议
GPL 3.0
