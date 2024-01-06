package main

import (
	"encoding/json"
	"fmt"
	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/app"
	"fyne.io/fyne/v2/canvas"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/data/binding"
	"fyne.io/fyne/v2/dialog"
	"fyne.io/fyne/v2/driver/desktop"
	"fyne.io/fyne/v2/layout"
	"fyne.io/fyne/v2/widget"
	"github.com/nicksnyder/go-i18n/v2/i18n"
	"image/color"
	"io"
	"net/http"
	"net/url"
	"os"
	"strconv"
	"strings"
	"time"
)

var mainWindow fyne.Window

var _fileLog *FetchLog

func bootGui() {
	logFile, err := os.OpenFile(AppExecDir()+"/fetch.log", os.O_WRONLY|os.O_APPEND|os.O_CREATE, 0666)
	if err != nil {
		_cliLog.Print(t(&i18n.Message{
			ID:    "LogCreatedFail",
			Other: "日志文件创建失败",
		}))
		return
	}
	_fileLog = &FetchLog{w: logFile}
	logoResource := getLogoResource()
	a := app.New()
	a.Settings().SetTheme(&fghGuiTheme{})

	mainWindow = a.NewWindow(fmt.Sprintf("Fetch Github Hosts - V%.1f", VERSION))
	mainWindow.Resize(fyne.NewSize(800, 580))
	mainWindow.SetIcon(logoResource)

	logoImage := canvas.NewImageFromResource(logoResource)
	logoImage.SetMinSize(fyne.NewSize(240, 240))
	tabs := container.NewAppTabs(
		container.NewTabItem(t(&i18n.Message{
			ID:    "ClientMode",
			Other: "客户端模式",
		}), guiClientMode()),
		container.NewTabItem(t(&i18n.Message{
			ID:    "ServerMode",
			Other: "服务端模式",
		}), guiServerMode()),
		container.NewTabItem(t(&i18n.Message{
			ID:    "About",
			Other: "关于",
		}), container.NewVBox(
			newMargin(fyne.NewSize(10, 10)),
			container.New(layout.NewCenterLayout(), logoImage),
			guiAbout(),
		)),
	)

	mainWindow.SetCloseIntercept(func() {
		mainWindow.Hide()
	})

	mainWindow.CenterOnScreen()
	mainWindow.SetContent(container.NewVBox(tabs))

	if err := GetCheckPermissionResult(); err != nil {
		time.AfterFunc(time.Second, func() {
			showAlert(err.Error())
		})
	}

	go checkVersion(nil)

	trayMenu := fyne.NewMenu("TrayMenu", fyne.NewMenuItem(t(&i18n.Message{
		ID:    "OpenHome",
		Other: "打开主界面",
	}), func() {
		mainWindow.Show()
	}))

	if desk, ok := a.(desktop.App); ok {
		desk.SetSystemTrayMenu(trayMenu)
		desk.SetSystemTrayIcon(logoResource)
	}

	mainWindow.ShowAndRun()
}

func getLogoResource() fyne.Resource {
	content, err := assetsFs.ReadFile("assets/public/logo.png")
	if err != nil {
		return nil
	}
	return &fyne.StaticResource{StaticName: "logo", StaticContent: content}
}

func getTicker(interval int) *time.Ticker {
	d := time.Minute
	if IsDebug() {
		d = time.Second
	}
	return time.NewTicker(d * time.Duration(interval))
}

func guiClientMode() (content fyne.CanvasObject) {
	logs, addFn := newLogScrollComponent(fyne.NewSize(800, 280))
	var cLog = NewFetchLog(NewGuiLogWriter(addFn))
	var startBtn, stopBtn *widget.Button
	var interval, customUrl, selectUrl = strconv.Itoa(_conf.Client.Interval), _conf.Client.CustomUrl, _conf.Client.SelectOrigin
	var isCustomOrigin bool
	intervalInput, urlInput := widget.NewEntryWithData(binding.BindString(&interval)), widget.NewEntryWithData(binding.BindString(&customUrl))
	var ticker *FetchTicker

	originSelectOpts := make([]string, 0, len(HostsOrigins))
	for k := range HostsOrigins {
		originSelectOpts = append(originSelectOpts, k)
	}

	originMethodOpts := []string{
		t(&i18n.Message{
			ID:    "HostsOptOfficial",
			Other: "官方指定hosts源",
		}),
		t(&i18n.Message{
			ID:    "HostsOptCustom",
			Other: "自定义hosts源",
		}),
	}

	originSelect := widget.NewSelect(originSelectOpts, func(s string) {
		_conf.Client.SelectOrigin = s
		selectUrl = HostsOrigins[s]
	})
	originSelect.Selected = _conf.Client.SelectOrigin
	selectUrl = HostsOrigins[originSelect.Selected]

	intervalForm := widget.NewFormItem(t(&i18n.Message{
		ID:    "GetIntervalMinutes",
		Other: "获取间隔（分钟）",
	}), intervalInput)
	originSelectForm := widget.NewForm(widget.NewFormItem(t(&i18n.Message{
		ID:    "HostsOrigin",
		Other: "Hosts源",
	}), originSelect))
	originCustomForm := widget.NewForm(widget.NewFormItem(t(&i18n.Message{
		ID:    "RemoteHostsUrl",
		Other: "远程Hosts链接",
	}), urlInput))

	if _conf.Client.Method == originMethodOpts[0] {
		originCustomForm.Hide()
	} else {
		originSelectForm.Hide()
	}

	var form *widget.Form
	originMethod := widget.NewRadioGroup(originMethodOpts, nil)
	originMethodForm := widget.NewFormItem(t(&i18n.Message{
		ID:    "HostsOrigin",
		Other: "Hosts源",
	}), originMethod)
	originMethod.OnChanged = func(s string) {
		_conf.Client.Method = s
		if s == originMethodOpts[0] {
			originCustomForm.Hide()
			originSelectForm.Show()
			isCustomOrigin = false
		} else {
			originSelectForm.Hide()
			originCustomForm.Show()
			isCustomOrigin = true
		}
	}

	originMethod.Selected = _conf.Client.Method

	form = widget.NewForm(
		intervalForm,
		originMethodForm,
	)

	startFetchExec := func() {
		intervalInt := parseStrIsNumberNotShowAlert(&interval, t(&i18n.Message{
			ID:    "GetIntervalNeedInt",
			Other: "获取间隔必须为整数",
		}))
		if intervalInt == nil {
			return
		}
		stopBtn.Enable()
		componentsStatusChange(false, startBtn, intervalInput, urlInput, originMethod, originSelect)
		ticker = NewFetchTicker(*intervalInt)
		if isCustomOrigin {
			go startClient(ticker, customUrl, cLog)
		} else {
			go startClient(ticker, selectUrl, cLog)
		}
		_conf.Client.CustomUrl = customUrl
		_conf.Client.Interval = *intervalInt
		_conf.Storage()
	}

	startBtn = widget.NewButton(t(&i18n.Message{
		ID:    "Start",
		Other: "启动",
	}), startFetchExec)
	stopBtn = widget.NewButton(t(&i18n.Message{
		ID:    "Stop",
		Other: "停止",
	}), func() {
		stopBtn.Disable()
		componentsStatusChange(true, startBtn, intervalInput, urlInput, originMethod, originSelect)
		ticker.Stop()
	})

	if _conf.Client.AutoFetch {
		startFetchExec()
		startBtn.Disable()
	} else {
		stopBtn.Disable()
	}
	autoFetchCheck := widget.NewCheck(t(&i18n.Message{
		ID:    "StartupAutoGet",
		Other: "启动软件自动获取",
	}), func(b bool) {
		if b != _conf.Client.AutoFetch {
			_conf.Client.AutoFetch = b
			_conf.Storage()
			showAlert(t(&i18n.Message{
				ID:    "StartupAutoGetTips",
				Other: "启动软件自动获取状态已改变，将会在下次启动程序时生效！",
			}))
		}
	})
	autoFetchCheck.SetChecked(_conf.Client.AutoFetch)

	buttons := container.New(layout.NewGridLayout(4), startBtn, stopBtn, widget.NewButton(t(&i18n.Message{
		ID:    "ClearHosts",
		Other: "清除hosts",
	}), func() {
		if err := flushCleanGithubHosts(); err != nil {
			showAlert(fmt.Sprintf("%s: %s", t(&i18n.Message{
				ID:    "CleanGithubHostsFail",
				Other: "清除hosts中的github记录失败",
			}), err.Error()))
		} else {
			showAlert(t(&i18n.Message{
				ID:    "CleanGithubHostsSuccess",
				Other: "hosts文件中的github记录已经清除成功",
			}))
		}
	}), container.New(layout.NewCenterLayout(), autoFetchCheck))
	margin := newMargin(fyne.NewSize(10, 10))
	return container.NewVBox(margin, form, originSelectForm, originCustomForm, margin, buttons,
		margin, logs)
}

func guiServerMode() (content fyne.CanvasObject) {
	logs, addFn := newLogScrollComponent(fyne.NewSize(800, 320))
	var sLog = NewFetchLog(NewGuiLogWriter(addFn))
	var startBtn, stopBtn *widget.Button
	var interval, port = strconv.Itoa(_conf.Server.Interval), strconv.Itoa(_conf.Server.Port)
	var ticker *FetchTicker
	intervalInput, portInput := widget.NewEntryWithData(binding.BindString(&interval)), widget.NewEntryWithData(binding.BindString(&port))
	statusLabel := widget.NewHyperlink(t(&i18n.Message{
		ID:    "ListeningAddressWait",
		Other: "监听地址：待启动",
	}), nil)
	startBtn = widget.NewButton(t(&i18n.Message{
		ID:    "Start",
		Other: "启动",
	}), func() {
		portInt := parseStrIsNumberNotShowAlert(&port, t(&i18n.Message{
			ID:    "PortMustBeInt",
			Other: "端口号必须为整数",
		}))
		if portInt == nil {
			return
		}
		intervalInt := parseStrIsNumberNotShowAlert(&interval, t(&i18n.Message{
			ID:    "GetIntervalNeedInt",
			Other: "获取间隔必须为整数",
		}))
		if intervalInt == nil {
			return
		}
		stopBtn.Enable()
		componentsStatusChange(false, startBtn, intervalInput, portInput)
		ticker = NewFetchTicker(*intervalInt)
		go startServer(ticker, *portInt, sLog)
		_conf.Server.Interval = *intervalInt
		_conf.Server.Port = *portInt
		_conf.Storage()
		listenerUrl := fmt.Sprintf("http://127.0.0.1:%d", *portInt)
		statusLabel.SetText(tf(&i18n.LocalizeConfig{
			DefaultMessage: &i18n.Message{
				ID:    "ListeningAddress",
				Other: "监听地址 {{.Addr}}",
			},
			TemplateData: map[string]interface{}{
				"Addr": listenerUrl,
			},
		}))
		statusLabel.SetURLFromString(listenerUrl)
	})
	stopBtn = widget.NewButton(t(&i18n.Message{
		ID:    "Stop",
		Other: "停止",
	}), func() {
		stopBtn.Disable()
		componentsStatusChange(true, startBtn, intervalInput, portInput)
		statusLabel.SetText(t(&i18n.Message{
			ID:    "ListeningAddressWait",
			Other: "监听地址：待启动",
		}))
		ticker.Stop()
	})
	stopBtn.Disable()
	form := widget.NewForm(
		widget.NewFormItem(t(&i18n.Message{
			ID:    "GetIntervalMinutes",
			Other: "获取间隔（分钟）",
		}), intervalInput),
		widget.NewFormItem(t(&i18n.Message{
			ID:    "StartupPort",
			Other: "启动端口号",
		}), portInput),
	)
	buttons := container.New(layout.NewGridLayout(2), startBtn, stopBtn)
	return container.NewVBox(newMargin(fyne.NewSize(10, 10)), form, buttons,
		container.New(layout.NewCenterLayout(), statusLabel),
		logs,
	)
}

func guiAbout() (content fyne.CanvasObject) {
	aboutNote := widget.NewRichTextFromMarkdown(fmt.Sprintf(`
%s
%s
`, t(&i18n.Message{
		ID: "AboutContent",
		Other: `# 介绍
Fetch Github Hosts是主要为解决研究及学习人员访问Github过慢或其他问题而提供的Github Hosts同步工具
---
# 开源协议
GNU General Public License v3.0

# 版本号`,
	}), fmt.Sprintf("V%.1f", VERSION)))
	for i := range aboutNote.Segments {
		if seg, ok := aboutNote.Segments[i].(*widget.TextSegment); ok {
			seg.Style.Alignment = fyne.TextAlignCenter
		}
		if seg, ok := aboutNote.Segments[i].(*widget.HyperlinkSegment); ok {
			seg.Alignment = fyne.TextAlignCenter
		}
	}
	github := widget.NewButton("Github", openUrl("https://github.com/Licoy/fetch-github-hosts"))
	feedback := widget.NewButton(t(&i18n.Message{
		ID:    "Feedback",
		Other: "反馈建议",
	}), openUrl("https://github.com/Licoy/fetch-github-hosts/issues"))
	var cv *widget.Button
	cv = widget.NewButton(t(&i18n.Message{
		ID:    "CheckUpdate",
		Other: "检查更新",
	}), func() {
		checkVersion(cv)
	})
	return container.NewVBox(aboutNote, container.New(layout.NewCenterLayout(), container.NewHBox(github, feedback, cv)))
}

func checkVersion(btn *widget.Button) {
	if btn != nil {
		btn.Disable()
		defer btn.Enable()
	}
	alertHandler := func(msg string) {
		if btn != nil {
			showAlert(msg)
		}
	}
	resp, err := http.Get("https://api.github.com/repos/Licoy/fetch-github-hosts/releases")
	if err != nil {
		alertHandler(fmt.Sprintf("%s: %s", t(&i18n.Message{
			ID:    "NetworkRequestFail",
			Other: "网络请求错误",
		}), err.Error()))
		return
	}
	if resp.StatusCode != http.StatusOK {
		alertHandler(fmt.Sprintf("%s: %s", t(&i18n.Message{
			ID:    "RequestFail",
			Other: "请求失败",
		}), err.Error()))
		return
	}
	all, err := io.ReadAll(resp.Body)
	if err != nil {
		alertHandler(fmt.Sprintf("%s: %s", t(&i18n.Message{
			ID:    "ReadUpdateResponseFail",
			Other: "读取更新响应内容失败",
		}), err.Error()))
		return
	}
	var releases []struct {
		TagName string `json:"tag_name"`
		HtmlUrl string `json:"html_url"`
	}
	if err = json.Unmarshal(all, &releases); err != nil {
		alertHandler(fmt.Sprintf("%s: %s", t(&i18n.Message{
			ID:    "ParseUpdateResponseFail",
			Other: "解析更新响应内容失败",
		}), err.Error()))
		return
	}
	if len(releases) == 0 {
		alertHandler(fmt.Sprintf("%s: %s", t(&i18n.Message{
			ID:    "CheckUpdateFail",
			Other: "检查更新失败",
		}), err.Error()))
		return
	}
	verStr := strings.Replace(strings.Replace(releases[0].TagName, "v", "", 1), "V", "", 1)
	float, err := strconv.ParseFloat(verStr, 64)
	if err != nil {
		alertHandler(fmt.Sprintf("%s: %s", t(&i18n.Message{
			ID:    "VersionParseFail",
			Other: "版本号解析失败",
		}), err.Error()))
		return
	}
	if VERSION >= float {
		alertHandler(t(&i18n.Message{
			ID:    "CurrentIsNewest",
			Other: "当前已是最新版本",
		}))
		return
	}
	confirm := dialog.NewConfirm(t(&i18n.Message{
		ID:    "UpdateTip",
		Other: "更新提示",
	}), t(&i18n.Message{
		ID:    "UpdateTipContent",
		Other: "检测到有新的版本，是否立即需要去下载最新版本？",
	}), func(b bool) {
		if b {
			openUrl(releases[0].HtmlUrl)()
		}
	}, mainWindow)
	confirm.SetDismissText(t(&i18n.Message{
		ID:    "UpdateLater",
		Other: "稍后更新",
	}))
	confirm.SetConfirmText(t(&i18n.Message{
		ID:    "DownloadNow",
		Other: "立即下载",
	}))
	confirm.Show()
}

func showAlert(msg string) {
	dialog.NewCustom(t(&i18n.Message{
		ID:    "Tip",
		Other: "提示",
	}), t(&i18n.Message{
		ID:    "Ok",
		Other: "确认",
	}), widget.NewLabel(msg), mainWindow).Show()
}

func parseStrIsNumberNotShowAlert(str *string, msg string) *int {
	res, err := strconv.Atoi(*str)
	if err != nil {
		showAlert(msg)
		return nil
	}
	return &res
}

func newLogScrollComponent(size fyne.Size) (scroll *container.Scroll, addFn func(string)) {
	var logs string
	textarea := widget.NewMultiLineEntry()
	textarea.Wrapping = fyne.TextWrapBreak
	textarea.Disable()
	scroll = container.NewScroll(textarea)
	scroll.SetMinSize(size)
	addFn = func(s string) {
		logs = s + logs
		textarea.SetText(logs)
	}
	return
}

func componentsStatusChange(enable bool, components ...fyne.Disableable) {
	for _, v := range components {
		if enable {
			v.Enable()
		} else {
			v.Disable()
		}
	}
}

func openUrl(urlStr string) func() {
	return func() {
		u, _ := url.Parse(urlStr)
		_ = fyne.CurrentApp().OpenURL(u)
	}
}

func newMargin(size fyne.Size) *canvas.Rectangle {
	margin := canvas.NewRectangle(color.Transparent)
	margin.SetMinSize(size)
	return margin
}
