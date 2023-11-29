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
)

func (w WebsiteLang) baseURL() string {
	if !w.IsChild {
		return "/"
	}

	return fmt.Sprintf("/%s/", w.Language.String())
}

func (w WebsiteLang) URLForHomePage() string {
	return w.baseURL()
}

func (w WebsiteLang) URLForBuiltin(b dd.Builtin) string {
	switch b {
	case dd.BuiltinFeed:
		return fmt.Sprintf("%s%s/", w.baseURL(), w.Feed.URLPrefix)
	case dd.BuiltinSearch:
		return fmt.Sprintf("%s%s/", w.baseURL(), search)
	case dd.BuiltinTags:
		return fmt.Sprintf("%s%s/", w.baseURL(), tags)
	default:
		return ""
	}
}

func (w WebsiteLang) URLForTag(t Tag) string {
	return fmt.Sprintf("%s%s/", w.URLForBuiltin(dd.BuiltinTags), t.Slug)
}

func (w WebsiteLang) URLForPageNote(slug string) string {
	return fmt.Sprintf("%s%s/", w.baseURL(), slug)
}

func (w WebsiteLang) URLForFeedNote(slug string) string {
	return fmt.Sprintf("%s%s/", w.URLForBuiltin(dd.BuiltinFeed), slug)
}

func (w WebsiteLang) URLForFile(file string) string {
	h := sha1.New()
	h.Write([]byte(file))
	filename := hex.EncodeToString(h.Sum(nil))
	extension := filepath.Ext(file)
	return fmt.Sprintf("%sfiles/%s%s", w.baseURL(), filename, extension)
}

func (w WebsiteLang) URLForSharedFile(file string) string {
	return fmt.Sprintf("/%s", file)
}

func (w WebsiteLang) AbsoluteURL(pattern string) string {
	return fmt.Sprintf("%s://%s%s", w.protocol(), w.Domain, pattern)
}

func (w WebsiteLang) protocol() string {
	if w.HTTPS {
		return "https"
	}

	return "http"
}
