package main

import (
	"embed"
	"fmt"
	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/app"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/data/binding"
	"fyne.io/fyne/v2/dialog"
	"fyne.io/fyne/v2/layout"
	"fyne.io/fyne/v2/widget"
	"io/ioutil"
	"os"
	"strconv"
	"time"
)

var mainWindow fyne.Window

//go:embed zcool-cryyt.ttf
var fyneFontEmbedFs embed.FS

func bootGui() {
	if err := initGuiFont(); err != nil {
		panic(err)
	}
	_ = os.Setenv("FYNE_FONT", AppExecDir()+"/"+GuiFontName)
	a := app.New()
	mainWindow = a.NewWindow("Fetch Github Hosts")
	mainWindow.Resize(fyne.NewSize(800, 580))
	mainWindow.SetFixedSize(true)

	tabs := container.NewAppTabs(
		container.NewTabItem("客户端模式", guiClientMode()),
		container.NewTabItem("服务端模式", guiServerMode()),
		container.NewTabItem("关于", guiAbout()),
	)

	mainWindow.SetContent(tabs)

	mainWindow.ShowAndRun()
	_ = os.Unsetenv("FYNE_FONT")
}

func getTicker(interval int) *time.Ticker {
	return time.NewTicker(time.Second * time.Duration(interval))
}

func guiClientMode() (content fyne.CanvasObject) {
	logs, addFn := newLogScrollComponent(fyne.NewSize(800, 260))
	var cLog = NewFetchLog(NewGuiLogWriter(addFn))
	var startBtn, stopBtn *widget.Button
	var interval, url, selectUrl = "60", "https://hosts.gitcdn.top/hosts.txt", ""
	var isCustomOrigin bool
	intervalInput, urlInput := widget.NewEntryWithData(binding.BindString(&interval)), widget.NewEntryWithData(binding.BindString(&url))
	var ticker *FetchTicker
	startBtn = widget.NewButton("启动", func() {
		intervalInt := parseStrIsNumberNotShowAlert(&interval, "获取间隔必须为整数")
		if intervalInt == nil {
			return
		}
		stopBtn.Enable()
		componentsStatusChange(false, startBtn, intervalInput, urlInput)
		ticker = NewFetchTicker(*intervalInt)
		if isCustomOrigin {
			go startClient(ticker, url, cLog)
		} else {
			go startClient(ticker, selectUrl, cLog)
		}

	})
	stopBtn = widget.NewButton("停止", func() {
		stopBtn.Disable()
		componentsStatusChange(true, startBtn, intervalInput, urlInput)
		ticker.Stop()
	})

	originSelectMapOpts := map[string]string{
		"FetchGithubHosts": "https://hosts.gitcdn.top/hosts.txt",
		"Github520":        "https://raw.hellogithub.com/hosts",
	}

	originSelectOpts := make([]string, 0, len(originSelectMapOpts))
	for k := range originSelectMapOpts {
		originSelectOpts = append(originSelectOpts, k)
	}

	originMethodOpts := []string{
		"官方指定hosts源",
		"自定义hosts源",
	}

	originSelect := widget.NewSelect(originSelectOpts, func(s string) {
		selectUrl = originSelectMapOpts[s]
	})
	originSelect.Selected = originSelectOpts[0]
	selectUrl = originSelectMapOpts[originSelect.Selected]

	intervalForm := widget.NewFormItem("获取间隔(分钟)", intervalInput)
	originSelectForm := widget.NewForm(widget.NewFormItem("hosts源", originSelect))

	originCustomForm := widget.NewForm(widget.NewFormItem("远程hosts链接", urlInput))
	originCustomForm.Hide()

	var form *widget.Form
	originMethod := widget.NewRadioGroup(originMethodOpts, nil)
	originMethodForm := widget.NewFormItem("远程hosts来源", originMethod)
	originMethod.OnChanged = func(s string) {
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

	originMethod.Selected = originMethodOpts[0]

	form = widget.NewForm(
		intervalForm,
		originMethodForm,
	)

	buttons := container.New(layout.NewGridLayout(2), startBtn, stopBtn)

	return container.NewVBox(widget.NewLabel(""), form, originSelectForm, originCustomForm, buttons, logs)
}

func guiServerMode() (content fyne.CanvasObject) {
	var startBtn, stopBtn *widget.Button
	var interval, port = "60", "9898"
	intervalInput, portInput := widget.NewEntryWithData(binding.BindString(&interval)), widget.NewEntryWithData(binding.BindString(&port))
	statusLabel := widget.NewLabel("监听地址：待启动")
	startBtn = widget.NewButton("启动", func() {
		portInt := parseStrIsNumberNotShowAlert(&port, "端口号必须为整数")
		if portInt == nil {
			return
		}
		intervalInt := parseStrIsNumberNotShowAlert(&interval, "获取间隔必须为整数")
		if intervalInt == nil {
			return
		}
		stopBtn.Enable()
		componentsStatusChange(false, startBtn, intervalInput, portInput)
		statusLabel.SetText(fmt.Sprintf("监听地址：http://127.0.0.1:%d", *portInt))
	})
	stopBtn = widget.NewButton("停止", func() {
		stopBtn.Disable()
		componentsStatusChange(true, startBtn, intervalInput, portInput)
		statusLabel.SetText("监听地址：待启动")
	})
	form := widget.NewForm(
		widget.NewFormItem("获取间隔(分钟)", intervalInput),
		widget.NewFormItem("启动端口号", portInput),
	)
	buttons := container.New(layout.NewGridLayout(2), startBtn, stopBtn)
	logs, _ := newLogScrollComponent(fyne.NewSize(800, 260))
	return container.NewVBox(widget.NewLabel(""), form, buttons,
		container.New(layout.NewCenterLayout(), statusLabel),
		logs,
	)
}

func guiAbout() (content fyne.CanvasObject) {
	return container.NewVBox(widget.NewLabel("about"))
}

func showAlert(msg string) {
	dialog.NewCustom("提示", "确认", widget.NewLabel(msg), mainWindow).Show()
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

func initGuiFont() (err error) {
	fontPath := AppExecDir() + "/" + GuiFontName
	_, err = os.Stat(fontPath)
	if err != nil {
		var file []byte
		file, err = fyneFontEmbedFs.ReadFile(GuiFontName)
		if err != nil {
			err = ComposeError("加载字体失败", err)
			return
		}
		if err = ioutil.WriteFile(fontPath, file, os.ModePerm); err != nil {
			err = ComposeError("加载所需字体文件失败", err)
			return
		}
	}
	return
}
