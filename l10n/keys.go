package l10n

import (
	"fmt"
)

type Key int

const (
	DateFormat = iota
	FooterPoweredBy
	TagsTitle
)

func (l *L10n) Str(key Key) string {
	switch key {
	case DateFormat:
		return l.loc.DateFormat
	case FooterPoweredBy:
		return l.loc.FooterPoweredBy
	case TagsTitle:
		return l.loc.TagsTitle
	default:
		panic(fmt.Sprintf("Invalid key '%d' in language", key))
	}
}
