package main

import (
	"errors"
	"fmt"
	"github.com/jessevdk/go-flags"
	"os"
)

type CmdArgs struct {
	Mode          string `default:"client" short:"m" long:"mode" description:"启动模式(client或server)"`
	FetchInterval int    `default:"60" short:"i" long:"interval" description:"获取hosts的间隔时间，单位为分钟"`
	Port          int    `default:"9898" short:"p" long:"port" description:"服务模式监听端口"`
	Url           string `default:"https://hosts.gitcdn.top/hosts.txt" short:"u" long:"url" description:"客户端模式远程hosts获取链接"`
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
		}
		panic(fmt.Sprintf("解析参数错误: %v", err))
	}
	if args.Version {
		fmt.Printf("版本号: V%.1f\n", VERSION)
	}
	if args.Mode != "client" && args.Mode != "server" {
		panic(fmt.Sprintf("无效的启动模式: %s", args.Mode))
	}
	if args.FetchInterval < 1 {
		panic(fmt.Sprintf("获取hosts的间隔时间不可以小于1分钟，当前为%d分钟", args.FetchInterval))
	}
	return args
}
