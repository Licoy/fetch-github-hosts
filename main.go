package main

import (
	"fmt"
	"io/ioutil"
	"net"
	"net/http"
	"os"
	"runtime"
	"time"
)

func main() {
	if !IsDebug() {
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
	}

	args := ParseBootArgs()
	logPrint(fmt.Sprintf("开始程序监听，当前以%d分钟更新一次Github-Hosts！", args.FetchInterval))
	logPrint("请不要关闭此窗口以保持再前台运行")
	logPrint("可以将此程序注册为服务，具体请参考项目说明：https://github.com/Licoy/fetch-github-hosts")

	ticker := time.NewTicker(time.Minute * time.Duration(args.FetchInterval))
	if args.Mode == "server" {
		startServer(ticker, args.Port)
	} else {
		startClient(ticker, args.Url)
	}
}

func startClient(ticker *time.Ticker, url string) {
	logPrint("远程hosts获取链接：" + url)
	fn := func() {
		if err := ClientFetchHosts(url); err != nil {
			logPrint("更新Github-Hosts失败：" + err.Error())
		} else {
			logPrint("更新Github-Hosts成功！")
		}
	}
	fn()
	for {
		select {
		case <-ticker.C:
			fn()
		}
	}
}

func startServer(ticker *time.Ticker, port int) {
	listen, err := net.Listen("tcp", fmt.Sprintf(":%d", port))
	if err != nil {
		fmt.Println("服务启动失败（可能是目标端口已被占用）：", err.Error())
		return
	}
	logPrint(fmt.Sprintf("已监听HTTP服务成功：http://127.0.0.1:%d", port))
	logPrint(fmt.Sprintf("hosts文件链接：http://127.0.0.1:%d/hosts.txt", port))
	logPrint(fmt.Sprintf("hosts的JSON格式链接：http://127.0.0.1:%d/hosts.json", port))
	go func() {
		if err = http.Serve(listen, &serverHandle{}); err != nil {
			fmt.Println("HTTP服务启动失败：", err.Error())
			os.Exit(1)
			return
		}
	}()
	fn := func() {
		if err := ServerFetchHosts(); err != nil {
			logPrint("执行更新Github-Hosts失败：" + err.Error())
		} else {
			logPrint("执行更新Github-Hosts成功！")
		}
	}
	fn()
	for {
		select {
		case <-ticker.C:
			fn()
		}
	}
}

func logPrint(msg string) {
	now := time.Now().Format("2006-01-02 15:04:05")
	fmt.Printf("[%s] %s\n", now, msg)
}

type serverHandle struct {
}

func (s serverHandle) ServeHTTP(resp http.ResponseWriter, request *http.Request) {
	p := request.URL.Path
	if p == "/" || p == "/hosts.txt" || p == "/hosts.json" {
		if p == "/" {
			p = "/index.html"
		}
		file, err := ioutil.ReadFile(AppExecDir() + p)
		if err != nil {
			resp.WriteHeader(http.StatusInternalServerError)
			resp.Write([]byte("server error"))
			logPrint("获取首页文件失败: " + err.Error())
			return
		}
		resp.Write(file)
		return
	}
	http.Redirect(resp, request, "/", http.StatusMovedPermanently)
}
