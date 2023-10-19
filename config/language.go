package config

import (
	"fmt"

	"github.com/karabatov/ddpub/config/internal/data"
	"github.com/karabatov/ddpub/dd"
)

type Language struct {
	// Language code.
	Code dd.Language
	// If true, the URL would be /en/, not /en-US/.
	UseShort bool
}

func (l Language) String() string {
	s := dd.SupportedLanguages[l.Code]
	if l.UseShort {
		return s.Short
	}
	return s.Full
}

func parseLanguage(d data.Language) (Language, error) {
	l := Language{UseShort: d.UseShort}

	// Return en-US if the code is not specified.
	if len(d.Full) == 0 {
		return l, nil
	}

	for code, s := range dd.SupportedLanguages {
		if s.Full == d.Full {
			l.Code = code
			return l, nil
		}
	}

	return l, fmt.Errorf("language '%s' not supported", d.Full)
}
