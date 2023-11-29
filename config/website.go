package config

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/karabatov/ddpub/dd"
)

// Website represents the configuration of a website.
type Website struct {
	Main       *WebsiteLang
	SubConfigs []*WebsiteLang
}

func NewWebsite(configDir string) (*Website, error) {
	var w Website

	// Read main config.

	cfgPath := configPath(configDir, dd.LanguageEnUS, false)
	mainConfig, err := newLang(cfgPath, dd.LanguageEnUS, false)
	if err != nil {
		return nil, err
	}
	w.Main = mainConfig

	if len(w.Main.Domain) == 0 {
		return nil, fmt.Errorf("domain field must be set in config file: %s", cfgPath)
	}

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

		// Overwrite domain and HTTPS setting.
		cfg.Domain = w.Main.Domain
		cfg.HTTPS = w.Main.HTTPS

		w.SubConfigs = append(w.SubConfigs, cfg)
	}

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
