package main

import (
	"errors"
	"fmt"
	"github.com/jessevdk/go-flags"
	"os"
)

type CmdArgs struct {
	Mode          string `short:"m" long:"mode" description:"启动模式(client或server)"`
	FetchInterval int    `default:"60" short:"i" long:"interval" description:"获取hosts的间隔时间，单位为分钟"`
	Port          int    `default:"9898" short:"p" long:"port" description:"服务模式监听端口"`
	Url           string `default:"https://hosts.gitcdn.top/hosts.txt" short:"u" long:"url" description:"客户端模式远程hosts获取链接"`
	Escalate      bool   `long:"escalate" description:"提权执行"`
	DontEscalate  bool   `long:"de" description:"禁止提权执行"`
	Version       bool   `short:"v" long:"version" description:"查看当前版本"`
}

func ParseBootArgs() *CmdArgs {
	args := &CmdArgs{}
	_, err := flags.ParseArgs(args, os.Args)
	if err != nil {
		et, y := err.(*flags.Error)
		if y {
			if errors.Is(flags.ErrHelp, et.Type) {
				os.Exit(0)
			}
			if !errors.Is(flags.ErrUnknownFlag, et.Type) {
				panic(fmt.Sprintf("解析参数错误: %v", err))
			}
		}
	}
	if args.Version {
		fmt.Printf("版本号: V%.1f\n", VERSION)
		os.Exit(0)
	}
	if args.Mode != "" && (args.Mode != "client" && args.Mode != "server") {
		fmt.Printf("无效的启动模式: %s，已自动设置为client\n", args.Mode)
		args.Mode = "client"
	}
	if args.FetchInterval < 1 {
		fmt.Printf("获取hosts的间隔时间不可以小于1分钟，当前为%d分钟，已自动设置为60分钟\n", args.FetchInterval)
		args.FetchInterval = 60
	}
	return args
}
