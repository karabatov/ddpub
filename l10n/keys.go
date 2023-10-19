package l10n

import (
	"fmt"

	"github.com/karabatov/ddpub/dd"
)

type Key int

const (
	FooterPoweredBy = iota
	TagsTitle
)

func (l *L10n) Str(key Key, lang dd.Language) string {
	s := l.loc[lang]
	switch key {
	case FooterPoweredBy:
		return s.FooterPoweredBy
	case TagsTitle:
		return s.TagsTitle
	default:
		panic(fmt.Sprintf("Invalid key '%d' in language '%s'", key, dd.SupportedLanguages[lang].Full))
	}
}
