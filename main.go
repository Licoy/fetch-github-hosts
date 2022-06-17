package main

import (
	"fmt"
	"runtime"
	"time"
)

func main() {
	permission, err := PreCheckHasHostsRWPermission()
	if err != nil {
		fmt.Println("检查hosts读写权限失败", err.Error())
		return
	}
	if !permission {
		if runtime.GOOS == Windows {
			fmt.Println("请鼠标右键选择【以管理员的身份运行】来执行本程序！")
		} else {
			fmt.Println("请以root账户或sudo来执行本程序！", err.Error())
		}
		return
	}
	args := ParseBootArgs()
	ticker := time.NewTicker(time.Minute * time.Duration(args.FetchInterval))
	logPrint(fmt.Sprintf("开始程序监听，当前以%d分钟更新一次Github-Hosts！", args.FetchInterval))
	logPrint("请不要关闭此窗口以保持再前台运行")
	logPrint("可以将此程序注册为服务，具体请参考项目说明：https://github.com/Licoy/fetch-github-hosts")
	if args.Mode == "server" {
		startServer(ticker)
	} else {
		startClient(ticker)
	}
}

func startServer(ticker *time.Ticker) {
	for {
		select {
		case <-ticker.C:
			if err := ServerFetchHosts(); err != nil {
				logPrint("执行更新Github-Hosts失败：" + err.Error())
			} else {
				logPrint("执行更新Github-Hosts成功！")
			}
		}
	}
}

func startClient(ticker *time.Ticker) {
	for {
		select {
		case <-ticker.C:
			if err := ClientFetchHosts(); err != nil {
				logPrint("更新Github-Hosts失败：" + err.Error())
			} else {
				logPrint("更新Github-Hosts成功！")
			}
		}
	}
}

func logPrint(msg string) {
	now := time.Now().Format("2006-01-02 15:04:05")
	fmt.Printf("[%s] %s\n", now, msg)
}
