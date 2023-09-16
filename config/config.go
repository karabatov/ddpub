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
	IDFromLink    dd.IDFromLinkFunc
}

func Load(configDir string) (Website, error) {
	var w Website

	cfg, err := readConfigFile(configDir)
	if err != nil {
		return w, err
	}

	noteIDValidator, err := makeNoteIDValidator(cfg.Notes.IdFormat)
	if err != nil {
		return w, err
	}
	w.IsValidNoteID = noteIDValidator

	idLinkExtractor, err := makeIDFromLinkFunc(cfg.Notes.IdLinkFormat, w.IsValidNoteID)
	if err != nil {
		return w, err
	}
	w.IDFromLink = idLinkExtractor

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

// Compile `id_format` regex and return validator func.
func makeNoteIDValidator(r string) (dd.NoteIDValidFunc, error) {
	// Load file ID regex from config and try to compile.
	validID, err := regexp.Compile(r)
	if err != nil {
		return nil, fmt.Errorf("could not compile regular expression '%s': %v", r, err)
	}

	return func(test string) bool {
		var id = validID.FindString(test)
		return len(id) > 0 && id == test
	}, nil
}

func makeIDFromLinkFunc(r string, isValid dd.NoteIDValidFunc) (dd.IDFromLinkFunc, error) {
	// Load ID link format regex from config and try to compile.
	idLinkFormat, err := regexp.Compile(r)
	if err != nil {
		return nil, fmt.Errorf("could not compile regular expression '%s': %v", r, err)
	}

	return func(link string) (dd.NoteID, bool) {
		id, ok := dd.FirstSubmatch(idLinkFormat, link)
		if !ok {
			return "", false
		}
		return dd.NoteID(id), isValid(id)
	}, nil
}
