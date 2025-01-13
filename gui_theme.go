//go:build !no_gui
// +build !no_gui

package main

import (
	"image/color"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/theme"
)

type fghGuiTheme struct {
}

func (f *fghGuiTheme) Font(s fyne.TextStyle) fyne.Resource {
	font, err := assetsFs.ReadFile("assets/" + GuiFontName)
	if err != nil {
		return theme.DefaultTheme().Font(s)
	}
	return fyne.NewStaticResource("fgh-font", font)
}

func (*fghGuiTheme) Color(c fyne.ThemeColorName, _ fyne.ThemeVariant) color.Color {
	switch c {
	case theme.ColorNamePrimary, theme.ColorNameButton:
		//#009966
		return color.RGBA{G: 0x99, B: 0x66, A: 0xff}
	case theme.ColorNameBackground:
		//#191b2c
		return color.RGBA{R: 0x19, G: 0x1b, B: 0x2c, A: 0xff}
	case theme.ColorNameMenuBackground, theme.ColorNameInputBackground, theme.ColorNameOverlayBackground:
		//#1f2437
		return color.RGBA{R: 0x1f, G: 0x24, B: 0x37, A: 0xff}
	case theme.ColorNameDisabledButton:
		//#629181
		return color.RGBA{R: 0x62, G: 0x91, B: 0x81, A: 0xff}
	case theme.ColorNameDisabled:
		//#34364a
		return color.RGBA{R: 0x34, G: 0x36, B: 0x4a, A: 0xff}
	default:
		return theme.DefaultTheme().Color(c, theme.VariantDark)
	}
}

func (*fghGuiTheme) Icon(n fyne.ThemeIconName) fyne.Resource {
	return theme.DefaultTheme().Icon(n)
}

func (*fghGuiTheme) Size(n fyne.ThemeSizeName) float32 {
	return theme.DefaultTheme().Size(n)
}
