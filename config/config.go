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
	IsValidNoteID dd.NoteIDValidFunc
	IDFromLink    dd.IDFromLinkFunc
	Homepage      Homepage
	Tags          map[dd.Tag]Tag
	Menu          []Menu
}

func (w Website) isTagPublished(tag dd.Tag) bool {
	_, ok := w.Tags[tag]
	return ok
}

func Load(configDir string) (Website, error) {
	var w Website

	cfg, err := readConfigFile(configDir)
	if err != nil {
		return w, err
	}

	w.IsValidNoteID, err = makeNoteIDValidator(cfg.Notes.IDFormat)
	if err != nil {
		return w, err
	}

	w.IDFromLink, err = makeIDFromLinkFunc(cfg.Notes.IDLinkFormat, w.IsValidNoteID)
	if err != nil {
		return w, err
	}

	w.Homepage, err = parseHomepage(cfg.Homepage, w.IsValidNoteID)
	if err != nil {
		return w, err
	}

	w.Tags, err = loadTags(cfg.Tags, w.IsValidNoteID)
	if err != nil {
		return w, err
	}

	for _, m := range cfg.Menu {
		menu, err := parseMenu(m, w.IsValidNoteID, w.isTagPublished)
		if err != nil {
			return w, err
		}
		w.Menu = append(w.Menu, menu)
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

func loadTags(s []data.Tag, isValid dd.NoteIDValidFunc) (map[dd.Tag]Tag, error) {
	tags := make(map[dd.Tag]Tag)
	for _, t := range s {
		tag, err := parseTag(t, isValid)
		if err != nil {
			return nil, err
		}
		if _, ok := tags[tag.Tag]; ok {
			return nil, fmt.Errorf("tag '%s' already published", tag.Tag)
		}
		tags[tag.Tag] = tag
	}
	return tags, nil
}
