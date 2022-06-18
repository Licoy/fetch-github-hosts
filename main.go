package main

import (
	"fmt"
	"os"
)

var _cliLog = &fetchLog{w: os.Stdout}

func main() {
	//if !IsDebug() {
	//	permission, err := PreCheckHasHostsRWPermission()
	//	if err != nil {
	//		fmt.Println("检查hosts读写权限失败", err.Error())
	//		return
	//	}
	//	if !permission {
	//		if runtime.GOOS == Windows {
	//			fmt.Println("请鼠标右键选择【以管理员的身份运行】来执行本程序！")
	//		} else {
	//			fmt.Println("请以root账户或sudo来执行本程序！", err.Error())
	//		}
	//		return
	//	}
	//}
	bootGui()
}

func bootCli() {
	args := ParseBootArgs()
	_cliLog.Print(fmt.Sprintf("开始程序监听，当前以%d分钟更新一次Github-Hosts！", args.FetchInterval))
	_cliLog.Print("请不要关闭此窗口以保持再前台运行")
	_cliLog.Print("可以将此程序注册为服务，具体请参考项目说明：https://github.com/Licoy/fetch-github-hosts")

	ticker := NewFetchTicker(args.FetchInterval)
	if args.Mode == "server" {
		startServer(ticker, args.Port, _cliLog)
	} else {
		startClient(ticker, args.Url, _cliLog)
	}
}
