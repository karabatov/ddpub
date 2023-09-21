package config

import (
	"fmt"

	"github.com/karabatov/ddpub/dd"
)

const (
	search = "search"
	tags   = "tags"
)

func (w Website) URLForBuiltin(b dd.Builtin) string {
	switch b {
	case dd.BuiltinFeed:
		return fmt.Sprintf("/%s/", w.Feed.URLPrefix)
	case dd.BuiltinSearch:
		return fmt.Sprintf("/%s/", search)
	case dd.BuiltinTags:
		return fmt.Sprintf("/%s/", tags)
	default:
		return ""
	}
}

func (w Website) URLForTag(t Tag) string {
	return fmt.Sprintf("%s%s/", w.URLForBuiltin(dd.BuiltinTags), t.Slug)
}

func (w Website) URLForMenuNote(slug string) string {
	return fmt.Sprintf("/%s/", slug)
}

func (w Website) URLForFeedNote(slug string) string {
	return fmt.Sprintf("%s%s/", w.URLForBuiltin(dd.BuiltinFeed), slug)
}
