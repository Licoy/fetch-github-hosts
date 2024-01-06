package main

import (
	"embed"
	"fmt"
	"github.com/getlantern/elevate"
	"github.com/nicksnyder/go-i18n/v2/i18n"
	"github.com/pelletier/go-toml/v2"
	"golang.org/x/text/language"
	"os"
	"runtime"
)

//go:embed assets
var assetsFs embed.FS

//go:embed active.*.toml
var localeFS embed.FS

var _cliLog = &FetchLog{w: os.Stdout}
var _local *i18n.Localizer
var _conf *FetchConf

func main() {
	args := ParseBootArgs()
	if !args.DontEscalate && !args.Escalate && runtime.GOOS != Linux {
		cmd := elevate.Command(os.Args[0], "--escalate")
		cmd.Run()
		os.Exit(0)
	}
	isGui := args.Mode == ""
	bundle := i18n.NewBundle(language.Chinese)
	bundle.RegisterUnmarshalFunc("toml", toml.Unmarshal)
	bundle.LoadMessageFileFS(localeFS, "active.en-US.toml")
	_conf = LoadFetchConf()
	_local = i18n.NewLocalizer(bundle, args.Lang, _conf.Lang)
	if isGui {
		bootGui()
	} else {
		bootCli(args)
	}
}

func bootCli(args *CmdArgs) {
	if !IsDebug() {
		if err := GetCheckPermissionResult(); err != nil {
			_cliLog.Print(err.Error())
			return
		}
	}
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
