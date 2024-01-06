package main

import (
	"embed"
	"errors"
	"fmt"
	"github.com/nicksnyder/go-i18n/v2/i18n"
	"io"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"syscall"
)

const (
	VERSION = 2.7
)

var (
	_debug    bool
	_execDir  string
	_logWrite io.Writer
)

func init() {
	_logWrite = os.Stdout
	_debug = os.Getenv("FETCH_GITHUB_HOST_DEBUG") != ""
	initAppExecDir()
}

func initAppExecDir() {
	if _debug {
		_execDir, _ = os.Getwd()
	} else {
		_exec, _ := os.Executable()
		_execDir = filepath.Dir(_exec)
	}
}

func IsDebug() bool {
	return _debug
}

func AppExecDir() string {
	return _execDir
}

func GetSystemHostsPath() string {
	switch runtime.GOOS {
	case Windows:
		return os.Getenv("SystemRoot") + "\\System32\\drivers\\etc\\hosts"
	}
	return "/etc/hosts"
}

func PreCheckHasHostsRWPermission() (yes bool, err error) {
	h, err := syscall.Open(GetSystemHostsPath(), syscall.O_RDWR, 0655)
	if err != nil {
		if strings.Contains(err.Error(), "Access is denied") {
			err = nil
		}
		return
	}
	syscall.Close(h)
	yes = true
	return
}

func ComposeError(msg string, err error) error {
	if err == nil {
		return errors.New(msg)
	}
	return errors.New(msg + ": " + err.Error())
}

func GetExecOrEmbedFile(fs *embed.FS, filename string) (template []byte, err error) {
	exeDirFile := AppExecDir() + "/" + filename
	_, err = os.Stat(exeDirFile)
	if err == nil {
		template, err = os.ReadFile(exeDirFile)
		return
	}
	template, err = fs.ReadFile(filename)
	return
}

func GetCheckPermissionResult() (err error) {
	permission, err := PreCheckHasHostsRWPermission()
	if err != nil {
		err = ComposeError(t(&i18n.Message{
			ID:    "PermissionCheckFail",
			Other: "检查hosts读写权限失败，请以sudo或管理员身份来运行本程序！",
		}), err)
		return
	}
	if !permission {
		if runtime.GOOS == Windows {
			err = errors.New(t(&i18n.Message{
				ID:    "RunAsAdminWin",
				Other: "请鼠标右键选择【以管理员的身份运行】来执行本程序！",
			}))
			fmt.Println(t(&i18n.Message{
				ID:    "RunAsAdminWin",
				Other: "请鼠标右键选择【以管理员的身份运行】来执行本程序！",
			}))
		} else {
			err = ComposeError(t(&i18n.Message{
				ID:    "RunAsAdminUnix",
				Other: "请以root账户或sudo来执行本程序！",
			}), err)
		}
	}
	return
}

func GetNewlineChar() string {
	if runtime.GOOS == Windows {
		return "\r\n"
	}
	return "\n"
}
