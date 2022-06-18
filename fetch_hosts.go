package main

import (
	"bytes"
	"embed"
	"encoding/json"
	"errors"
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
)

//go:embed index.template
var indexTemplate embed.FS

//go:embed domains.json
var domainsJson embed.FS

func startClient(ticker *FetchTicker, url string, flog *fetchLog) {
	flog.Print("远程hosts获取链接：" + url)
	fn := func() {
		if err := ClientFetchHosts(url); err != nil {
			flog.Print("更新Github-Hosts失败：" + err.Error())
		} else {
			flog.Print("更新Github-Hosts成功！")
		}
	}
	fn()
	for {
		select {
		case <-ticker.Ticker.C:
			fn()
		case <-ticker.CloseChan:
			fmt.Println("close")
			return
		}
	}
}

func startServer(ticker *FetchTicker, port int, flog *fetchLog) {
	listen, err := net.Listen("tcp", fmt.Sprintf(":%d", port))
	if err != nil {
		fmt.Println("服务启动失败（可能是目标端口已被占用）：", err.Error())
		return
	}
	flog.Print(fmt.Sprintf("已监听HTTP服务成功：http://127.0.0.1:%d", port))
	flog.Print(fmt.Sprintf("hosts文件链接：http://127.0.0.1:%d/hosts.txt", port))
	flog.Print(fmt.Sprintf("hosts的JSON格式链接：http://127.0.0.1:%d/hosts.json", port))
	go func() {
		if err = http.Serve(listen, &serverHandle{flog}); err != nil {
			fmt.Println("HTTP服务启动失败：", err.Error())
			os.Exit(1)
			return
		}
	}()
	fn := func() {
		if err := ServerFetchHosts(); err != nil {
			flog.Print("执行更新Github-Hosts失败：" + err.Error())
		} else {
			flog.Print("执行更新Github-Hosts成功！")
		}
	}
	fn()
	for {
		select {
		case <-ticker.Ticker.C:
			fn()
		}
	}
}

type serverHandle struct {
	flog *fetchLog
}

func (s *serverHandle) ServeHTTP(resp http.ResponseWriter, request *http.Request) {
	p := request.URL.Path
	if p == "/" || p == "/hosts.txt" || p == "/hosts.json" {
		if p == "/" {
			p = "/index.html"
		}
		file, err := ioutil.ReadFile(AppExecDir() + p)
		if err != nil {
			resp.WriteHeader(http.StatusInternalServerError)
			resp.Write([]byte("server error"))
			s.flog.Print("获取首页文件失败: " + err.Error())
			return
		}
		resp.Write(file)
		return
	}
	http.Redirect(resp, request, "/", http.StatusMovedPermanently)
}

// ClientFetchHosts 获取最新的host并写入hosts文件
func ClientFetchHosts(url string) (err error) {
	hostsPath := GetSystemHostsPath()
	hostsBytes, err := ioutil.ReadFile(hostsPath)
	if err != nil {
		err = ComposeError("读取文件hosts错误", err)
		return
	}

	resp, err := http.Get(url)
	if err != nil || resp.StatusCode != http.StatusOK {
		err = ComposeError("获取最新的hosts失败", err)
		return
	}

	fetchHosts, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		err = ComposeError("读取最新的hosts失败", err)
		return
	}

	fetchHostsStr := strings.Trim(string(fetchHosts), "\n")

	mth, err := regexp.Compile(`# fetch-github-hosts begin(([\s\S])*.?)# fetch-github-hosts end`)
	if err != nil {
		err = ComposeError("创建内容正则匹配失败", err)
		return
	}

	if len(mth.FindStringSubmatch(fetchHostsStr)) == 0 {
		err = errors.New("无效的远程hosts链接，未通过格式校验")
		return
	}

	hosts := string(hostsBytes)

	findStr := mth.FindStringSubmatch(hosts)
	if len(findStr) > 0 {
		hosts = strings.Replace(hosts, findStr[0], fetchHostsStr, 1)
	} else {
		hosts += "\n\n" + fetchHostsStr + "\n"
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
	fileData, err := GetExecOrEmbedFile(&domainsJson, "domains.json")
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
	templateFile, err = GetExecOrEmbedFile(&indexTemplate, "index.template")
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
