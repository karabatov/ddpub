package config

import (
	"fmt"

	"github.com/karabatov/ddpub/config/internal/data"
	"github.com/karabatov/ddpub/dd"
)

type lang struct {
	num   dd.Language
	code  string
	short string
}

var supportedLanguages = map[dd.Language]lang{
	dd.LanguageEnUS: {dd.LanguageEnUS, "en-US", "en"},
	dd.LanguageEnUK: {dd.LanguageEnUK, "en-UK", "en"},
	dd.LanguageRuRU: {dd.LanguageRuRU, "ru-RU", "ru"},
}

type Language struct {
	// Language code.
	Code dd.Language
	// If true, the URL would be /en/, not /en-US/.
	Short bool
}

func (l Language) String() string {
	s := supportedLanguages[l.Code]
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
			l.Code = s.num
			return l, nil
		}
	}

	return l, fmt.Errorf("language '%s' not supported", d.Code)
}
