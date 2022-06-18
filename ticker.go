package main

import "time"

type FetchTicker struct {
	Ticker    *time.Ticker
	CloseChan chan struct{}
}

func NewFetchTicker(interval int) *FetchTicker {
	return &FetchTicker{getTicker(interval), make(chan struct{})}
}

func (f *FetchTicker) Stop() {
	f.Ticker.Stop()
	close(f.CloseChan)
}
