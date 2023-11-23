package config

import (
	"crypto/sha1"
	"encoding/hex"
	"fmt"
	"path/filepath"

	"github.com/karabatov/ddpub/dd"
)

const (
	search = "search"
	tags   = "tags"
	theme  = "theme.css"
)

func (w WebsiteLang) URLForBuiltin(b dd.Builtin) string {
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

func (w WebsiteLang) URLForTag(t Tag) string {
	return fmt.Sprintf("%s%s/", w.URLForBuiltin(dd.BuiltinTags), t.Slug)
}

func (w WebsiteLang) URLForPageNote(slug string) string {
	return fmt.Sprintf("/%s/", slug)
}

func (w WebsiteLang) URLForFeedNote(slug string) string {
	return fmt.Sprintf("%s%s/", w.URLForBuiltin(dd.BuiltinFeed), slug)
}

func (w WebsiteLang) URLForThemeCSS() string {
	return "/" + theme
}

func (w WebsiteLang) URLForFile(file string) string {
	h := sha1.New()
	h.Write([]byte(file))
	filename := hex.EncodeToString(h.Sum(nil))
	extension := filepath.Ext(file)
	return fmt.Sprintf("/files/%s%s", filename, extension)
}
