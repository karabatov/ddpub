package l10n

import (
	"embed"
	"fmt"

	"github.com/karabatov/ddpub/dd"
	"github.com/karabatov/ddpub/l10n/internal/strings"
)

var (
	//go:embed strings/strings.*.toml
	stringsFiles embed.FS
)

type L10n struct {
	loc map[dd.Language]strings.Strings
}

func New() (*L10n, error) {
	return nil, fmt.Errorf("not loaded")
}

func loadLanguage(l dd.Language, fs *embed.FS) (strings.Strings, error) {
	var s strings.Strings
	return s, nil
}
