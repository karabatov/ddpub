package l10n

import (
	"fmt"

	"github.com/karabatov/ddpub/dd"
)

type Key int

const (
	DateFormat = iota
	FooterPoweredBy
	TagsTitle
)

func (l *L10n) Str(key Key, lang dd.Language) string {
	s := l.loc[lang]
	switch key {
	case DateFormat:
		return s.DateFormat
	case FooterPoweredBy:
		return s.FooterPoweredBy
	case TagsTitle:
		return s.TagsTitle
	default:
		panic(fmt.Sprintf("Invalid key '%d' in language '%s'", key, dd.SupportedLanguages[lang].Full))
	}
}
