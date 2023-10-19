package l10n

import (
	"bytes"
	"embed"
	"fmt"

	"github.com/karabatov/ddpub/dd"
	"github.com/karabatov/ddpub/l10n/internal/strings"
	"github.com/pelletier/go-toml/v2"
)

var (
	//go:embed strings/strings.*.toml
	stringsFiles embed.FS
)

type L10n struct {
	loc strings.Strings
}

func New(lang dd.Language) (*L10n, error) {
	var l L10n
	s, err := loadLanguage(lang, &stringsFiles)
	if err != nil {
		return nil, err
	}
	l.loc = s
	return &l, nil
}

func loadLanguage(l dd.Language, fs *embed.FS) (strings.Strings, error) {
	langCode := dd.SupportedLanguages[l]
	file, err := fs.ReadFile(fmt.Sprintf("strings/strings.%s.toml", langCode.Full))
	if err != nil {
		return strings.Strings{}, err
	}

	decoder := toml.NewDecoder(bytes.NewReader(file))
	decoder.DisallowUnknownFields()

	var s strings.Strings
	err = decoder.Decode(&s)
	if err != nil {
		return strings.Strings{}, err
	}
	return s, nil
}
