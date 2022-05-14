## 介绍
`fetch-github-hosts`是主要为解决研究及学习人员访问`Github`过慢或其他问题而提供的免费的`Github hosts`同步服务。

本项目部分参考于 [Github520](https://github.com/521xueweihan/GitHub520) ，
但与之不同的是前者是通过`ipaddress.com`获取`github.com`的`hosts`， 
而此项目是通过部署本身的服务器来获取`github.com`的`hosts`，所以在IP节点上会存在一定的差异。

## 使用方法
### 手动
#### 添加hosts
访问 [https://hosts.gitcdn.top/hosts.txt](https://hosts.gitcdn.top/hosts.txt) ，
将其全部内容粘贴到你的hosts文件中，即可。
- `Linux / MacOS` hosts路径：`/etc/hosts`
- `Windows` hosts路径：`C:\Windows\System32\drivers\etc\hosts`
#### 刷新生效
- `Linux`: `/etc/init.d/network restart`
- `Windows`: `ipconfig /flushdns`
- `Macos`: `sudo killall -HUP mDNSResponder`
### Unix/Linux 一键使用
```shell
sed -i "/# fetch-github-hosts begin/Q" /etc/hosts && curl https://hosts.gitcdn.top/hosts.txt >> /etc/hosts
```
> 提示：可以设置crontab定时任务定时获取更新即可，解放双手！

### Chrome
使用 [FasterHosts](https://github.com/gauseen/faster-hosts) 插件，若访问速度过慢可以直接使用
[点击此处](https://gitcdn.top/https://github.com/gauseen/faster-hosts/archive/refs/heads/master.zip) 来进行下载。

下载完成之后解压压缩包，Chrome地址栏输入`chrome://extensions/`回车进入，勾选`开发者模式`，选择`加载已解压的扩展程序`，
选择刚才的解压目录即可。
### Windows /MacOS 及其他桌面端
使用 [SwitchHosts](https://swh.app/) 桌面端应用，安装添加新规则：
- `Title`: 任意
- `Type`: `Remote`
- `Url`: `https://hosts.gitcdn.top/hosts.txt`
- `Auto refresh`: `1 hour`

## 私有部署
下载本仓库的代码：[fetch-github-hosts.zip](https://gitcdn.top/https://github.com/Licoy/fetch-github-hosts/archive/refs/heads/main.zip) ，
部署到任意一个含有PHP环境的服务器即可，部署完成之后可以计划任务脚本定时更新hosts：
```shell
cd /wwwroot/fetch-github-hosts #此处更换为你部署的项目路径
php fetch_hosts.php
```
> 注意：必须部署到非大陆的服务器节点！

### 开源协议
[GPL 3.0](./LICENSE)
