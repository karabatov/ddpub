package config

import (
	_ "embed"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/karabatov/ddpub/dd"
)

//go:embed files/theme.css
var themeCSS []byte

//go:embed files/favicon.ico
var faviconFile []byte

// Website represents the configuration of a website.
type Website struct {
	sharedFiles []SharedFile
	Main        *WebsiteLang
	SubConfigs  []*WebsiteLang
}

func NewWebsite(configDir string) (*Website, error) {
	var w Website

	// Read shared files.
	w.sharedFiles = []SharedFile{
		{
			Filename:    "theme.css",
			Content:     themeCSS,
			ContentType: "text/css",
		},
		{
			Filename:    "favicon.ico",
			Content:     faviconFile,
			ContentType: "image/svg+xml",
		},
	}
	// If there are any of the named files present in the config dir, overload them.
	for i := range w.sharedFiles {
		w.sharedFiles[i].overload(configDir)
	}

	// Read main config.

	cfgPath := configPath(configDir, dd.LanguageEnUS, false)
	mainConfig, err := newLang(cfgPath, dd.LanguageEnUS, false)
	if err != nil {
		return nil, err
	}
	w.Main = mainConfig
	w.Main.SharedFiles = w.sharedFiles

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
