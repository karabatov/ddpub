package config

import (
	"fmt"

	"github.com/karabatov/ddpub/dd"
)

func (w Website) URLForBuiltin(b dd.Builtin) string {
	switch b {
	case dd.BuiltinFeed:
		return fmt.Sprintf("/%s/", w.Feed.URLPrefix)
	case dd.BuiltinSearch:
		return "/search/"
	case dd.BuiltinTags:
		return "/tags/"
	default:
		return ""
	}
}
