package config

import (
	_ "embed"
	"errors"
	"os"
	"path/filepath"
	"strings"

	"github.com/karabatov/ddpub/dd"
)

//go:embed theme.css
var themeCSS []byte

// Website represents the configuration of a website.
type Website struct {
	Main       *WebsiteLang
	SubConfigs []*WebsiteLang
	ThemeCSS   []byte
}

func New(configDir string) (*Website, error) {
	var w Website

	// Read main config.

	cfgPath := configPath(configDir, dd.LanguageEnUS, false)
	mainConfig, err := newLang(cfgPath, dd.LanguageEnUS, false)
	if err != nil {
		return nil, err
	}
	w.Main = mainConfig

	// Read language subconfigs.

	w.SubConfigs = make([]*WebsiteLang, 0)
	for lang := range dd.SupportedLanguages {
		// Skip the "main" language.
		if lang == mainConfig.Language.Code {
			continue
		}

		cfgPath = configPath(configDir, lang, true)

		// Ignore any configs that are missing.
		if !configExists(cfgPath) {
			continue
		}

		cfg, err := newLang(cfgPath, lang, true)
		if err != nil {
			return nil, err
		}
		w.SubConfigs = append(w.SubConfigs, cfg)
	}

	w.ThemeCSS = themeCSS

	return &w, nil
}

func configExists(configPath string) bool {
	if _, err := os.Stat(configPath); errors.Is(err, os.ErrNotExist) {
		return false
	}

	return true
}

func configPath(configDir string, lang dd.Language, isChild bool) string {
	configDir = filepath.Clean(configDir)
	configName := []string{"config"}
	if isChild {
		configName = append(configName, dd.SupportedLanguages[lang].Full)
	}
	configName = append(configName, "toml")
	return filepath.Join(configDir, strings.Join(configName, "."))
}
