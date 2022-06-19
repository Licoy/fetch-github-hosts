package main

import (
	"github.com/spf13/viper"
)

type FetchConf struct {
	Client struct {
		Interval     int
		Method       string
		SelectOrigin string
		CustomUrl    string
	}
	Server struct {
		Interval int
		Port     int
	}
}

func (f *FetchConf) Storage() {
	viper.Set("client.interval", f.Client.Interval)
	viper.Set("client.method", f.Client.Method)
	viper.Set("client.selectorigin", f.Client.SelectOrigin)
	viper.Set("client.customurl", f.Client.CustomUrl)
	viper.Set("server.interval", f.Server.Interval)
	viper.Set("server.port", f.Server.Port)
	if err := viper.WriteConfigAs("conf.yaml"); err != nil {
		showAlert("持久化配置信息失败：" + err.Error())
	}
}

func LoadFetchConf() *FetchConf {
	viper.AddConfigPath(AppExecDir())
	viper.SetConfigName("conf")
	viper.SetConfigType("yaml")
	viper.SetDefault("client.interval", 60)
	viper.SetDefault("client.method", "官方指定hosts源")
	viper.SetDefault("client.selectorigin", "FetchGithubHosts")
	viper.SetDefault("server.interval", 60)
	viper.SetDefault("server.port", 9898)
	var fileNotExits bool
	if err := viper.ReadInConfig(); err != nil {
		if _, ok := err.(viper.ConfigFileNotFoundError); !ok {
			showAlert("加载配置文件错误")
		} else {
			fileNotExits = true
		}
	}
	res := FetchConf{}
	if err := viper.Unmarshal(&res); err != nil {
		showAlert("配置文件解析失败")
	}
	if fileNotExits {
		res.Storage()
	}
	return &res
}
