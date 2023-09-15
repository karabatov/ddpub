// Package config loads and validates website config from a directory.
package config

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"

	"github.com/BurntSushi/toml"
	"github.com/karabatov/ddpub/config/internal/data"
	"github.com/karabatov/ddpub/dd"
)

// Website represents the configuration of a website.
type Website struct {
	Homepage      Homepage
	Menu          []Menu
	IsValidNoteID dd.NoteIDValidFunc
}

func Load(configDir string) (Website, error) {
	var w Website

	cfg, err := readConfigFile(configDir)
	if err != nil {
		return w, err
	}

	// Load file ID regex from config and try to compile.
	validID, err := regexp.Compile(cfg.Notes.IdFormat)
	if err != nil {
		return w, fmt.Errorf("could not compile id_format regular expression '%s': %v", cfg.Notes.IdFormat, err)
	}

	w.IsValidNoteID = func(test string) bool {
		var id = validID.FindString(test)
		return len(id) > 0 && id == test
	}

	return w, nil
}

func readConfigFile(configDir string) (data.ConfigFile, error) {
	configDir = filepath.Clean(configDir)
	configPath := filepath.Join(configDir, "config.toml")
	configFile, err := os.ReadFile(configPath)
	if err != nil {
		return data.ConfigFile{}, fmt.Errorf("could not open config file '%s': %v", configPath, err)
	}

	var cfg data.ConfigFile
	err = toml.Unmarshal(configFile, &cfg)
	if err != nil {
		return data.ConfigFile{}, fmt.Errorf("could not load config file '%s': %v", configPath, err)
	}

	return cfg, nil
}
