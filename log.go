package main

import (
	"fmt"
	"io"
	"time"
)

type FetchLog struct {
	w io.Writer
}

func NewFetchLog(w io.Writer) *FetchLog {
	return &FetchLog{w}
}

func (f *FetchLog) Print(msg string) {
	now := time.Now().Format("2006-01-02 15:04:05")
	fmt.Fprintf(f.w, "[%s] %s\n", now, msg)
}

type guiLogWriter struct {
	addFn func(string)
}

func NewGuiLogWriter(addFn func(string)) io.Writer {
	return &guiLogWriter{addFn}
}

func (g *guiLogWriter) Write(p []byte) (n int, err error) {
	g.addFn(string(p))
	return len(p), nil
}
