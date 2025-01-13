//go:build no_gui
// +build no_gui

package main

import (
	"fmt"
	"time"
)

var _fileLog *FetchLog

func bootGui() {
	fmt.Println("GUI is not supported in this build")
	fmt.Println("Please use the command line interface")
	fmt.Println("For example: sudo ./fetch-github-hosts -m client ")
}

func getTicker(interval int) *time.Ticker {
	d := time.Minute
	if IsDebug() {
		d = time.Second
	}
	return time.NewTicker(d * time.Duration(interval))
}
