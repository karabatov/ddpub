package config

import (
	"fmt"

	"github.com/karabatov/ddpub/config/internal/data"
)

type LangNumber int

const (
	LanguageEnUS = iota
	LanguageEnUK
	LanguageRuRU
)

type lang struct {
	num   LangNumber
	code  string
	short string
}

var supportedLanguages = map[LangNumber]lang{
	LanguageEnUS: {LanguageEnUS, "en-US", "en"},
	LanguageEnUK: {LanguageEnUK, "en-UK", "en"},
	LanguageRuRU: {LanguageRuRU, "ru-RU", "ru"},
}

type Language struct {
	// Language code.
	Number LangNumber
	// If true, the URL would be /en/, not /en-US/.
	Short bool
}

func (l Language) Code() string {
	s := supportedLanguages[l.Number]
	if l.Short {
		return s.short
	}
	return s.code
}

func parseLanguage(d data.Language) (Language, error) {
	l := Language{Short: d.Short}

	// Return en-US if the code is not specified.
	if len(d.Code) == 0 {
		return l, nil
	}

	for _, s := range supportedLanguages {
		if s.code == d.Code {
			l.Number = s.num
			return l, nil
		}
	}

	return l, fmt.Errorf("language '%s' not supported", d.Code)
}
