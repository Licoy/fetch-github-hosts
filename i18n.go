package main

import (
	"fmt"
	"github.com/nicksnyder/go-i18n/v2/i18n"
)

func tf(cfg *i18n.LocalizeConfig) string {
	localize, err := _local.Localize(cfg)
	if err != nil {
		fmt.Println(err)
		if cfg.DefaultMessage.One != "" {
			return cfg.DefaultMessage.One
		}
		return cfg.DefaultMessage.Other
	}
	return localize
}

func tfs(msg *i18n.Message, params map[string]interface{}) string {
	return tf(&i18n.LocalizeConfig{
		DefaultMessage: msg,
		TemplateData:   params,
	})
}

func t(msg *i18n.Message) string {
	return tf(&i18n.LocalizeConfig{
		DefaultMessage: msg,
	})
}
