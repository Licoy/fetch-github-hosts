package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net"
	"net/http"
	"os"
	"regexp"
	"strings"
	"time"
)

const (
	Windows = "windows"
	Linux   = "linux"
	Darwin  = "darwin"
	HostUrl = "https://hosts.gitcdn.top/hosts.txt"
)

// ClientFetchHosts 获取最新的host并写入hosts文件
func ClientFetchHosts() (err error) {
	hostsPath := GetSystemHostsPath()
	hostsBytes, err := ioutil.ReadFile(hostsPath)
	if err != nil {
		err = ComposeError("读取文件hosts错误", err)
		return
	}

	resp, err := http.Get(HostUrl)
	if err != nil || resp.StatusCode != http.StatusOK {
		err = ComposeError("获取最新的hosts失败", err)
		return
	}

	fetchHosts, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		err = ComposeError("读取最新的hosts失败", err)
		return
	}

	hosts := string(hostsBytes)

	mth, err := regexp.Compile(`# fetch-github-hosts begin(([\s\S])*.?)# fetch-github-hosts end`)
	if err != nil {
		err = ComposeError("创建内容正则匹配失败", err)
		return
	}
	findStr := mth.FindStringSubmatch(hosts)
	if len(findStr) > 0 {
		hosts = strings.Replace(hosts, findStr[0], string(fetchHosts), 1)
	} else {
		hosts += "\n\n" + string(fetchHosts)
	}

	if err = ioutil.WriteFile(hostsPath, []byte(hosts), os.ModeType); err != nil {
		err = ComposeError("写入hosts文件失败，请用超级管理员身份启动本程序！", err)
		return
	}

	return
}

// ServerFetchHosts 服务端获取github最新的hosts并写入到对应文件及更新首页
func ServerFetchHosts() (err error) {
	execDir := AppExecDir()
	fileData, err := ioutil.ReadFile(execDir + "/domains.json")
	if err != nil {
		err = ComposeError("读取文件domains.json错误", err)
		return
	}

	var domains []string
	if err = json.Unmarshal(fileData, &domains); err != nil {
		err = ComposeError("domain.json解析失败", err)
		return
	}

	hostJson, hostFile, now, err := FetchHosts(domains)
	if err != nil {
		err = ComposeError("获取Github的Host失败", err)
		return
	}

	if err = ioutil.WriteFile(execDir+"/hosts.json", hostJson, 0775); err != nil {
		err = ComposeError("写入数据到hosts.json文件失败", err)
		return
	}

	if err = ioutil.WriteFile(execDir+"/hosts.txt", hostFile, 0775); err != nil {
		err = ComposeError("写入数据到hosts.txt文件失败", err)
		return
	}

	var templateFile []byte
	templateFile, err = ioutil.ReadFile(execDir + "/index-template.html")
	if err != nil {
		err = ComposeError("读取首页模板文件失败", err)
		return
	}

	templateData := strings.Replace(string(templateFile), "<!--time-->", now, 1)
	if err = ioutil.WriteFile(execDir+"/index.html", []byte(templateData), 0775); err != nil {
		err = ComposeError("写入更新信息到首页文件失败", err)
		return
	}

	return
}

func FetchHosts(domains []string) (hostsJson, hostsFile []byte, now string, err error) {
	now = time.Now().Format("2006-01-02 15:04:05")
	hosts := make([][]string, 0, len(domains))
	hostsFileData := bytes.NewBufferString("# fetch-github-hosts begin\n")
	for _, domain := range domains {
		host, err := net.LookupHost(domain)
		if err != nil {
			fmt.Println("获取主机记录失败: ", err.Error())
			continue
		}
		item := []string{host[0], domain}
		hosts = append(hosts, item)
		hostsFileData.WriteString(fmt.Sprintf("%-28s%s\n", item[0], item[1]))
	}
	hostsFileData.WriteString("# last fetch time: ")
	hostsFileData.WriteString(now)
	hostsFileData.WriteString("\n# update url: https://hosts.gitcdn.top/hosts.txt\n# fetch-github-hosts end\n\n")
	hostsFile = hostsFileData.Bytes()
	hostsJson, err = json.Marshal(hosts)
	return
}
