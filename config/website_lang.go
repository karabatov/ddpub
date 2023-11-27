// Package config loads and validates website config from a directory.
package config

import (
	_ "embed"
	"fmt"
	"os"
	"regexp"
	"sort"

	"github.com/karabatov/ddpub/config/internal/data"
	"github.com/karabatov/ddpub/dd"
	"github.com/karabatov/ddpub/l10n"
	"github.com/pelletier/go-toml/v2"
)

//go:embed theme.css
var themeCSS []byte

//go:embed favicon.ico
var faviconFile []byte

// WebsiteLang represents the configuration of one language of a website.
type WebsiteLang struct {
	// IsChild is true if this is not the main, default config.
	IsChild       bool
	Domain        string
	HTTPS         bool
	Twitter       string
	Title         string
	IsValidNoteID dd.NoteIDValidFunc
	IDFromLink    dd.IDFromLinkFunc
	IDFromFile    dd.IDFromFileFunc
	Homepage      Homepage
	Language      Language
	Tags          map[dd.Tag]Tag
	Menu          []Menu
	Feed          Feed
	Pages         Pages
	ThemeCSS      []byte
	Favicon       []byte
	FaviconType   string
	HeadSuffix    string
	NoteSuffix    string
	FooterPrefix  string
	localizer     *l10n.L10n
}

func (w WebsiteLang) isTagPublished(tag dd.Tag) bool {
	_, ok := w.Tags[tag]
	return ok
}

func newLang(configPath string, lang dd.Language, isChild bool) (*WebsiteLang, error) {
	var w WebsiteLang

	cfg, err := readConfigFile(configPath)
	if err != nil {
		return nil, err
	}

	w.Domain = cfg.Domain
	w.HTTPS = cfg.HTTPS
	w.Twitter = cfg.Twitter

	w.IsChild = isChild

	w.Title = cfg.Title

	w.Language, err = parseLanguage(cfg.Language)
	if err != nil {
		return nil, err
	}

	if isChild && w.Language.Code != lang {
		return nil, fmt.Errorf("mismatched language in config: %s", w.Language.String())
	}

	w.localizer, err = l10n.New(w.Language.Code)
	if err != nil {
		return nil, err
	}

	w.IsValidNoteID, err = makeNoteIDValidator(cfg.Notes.IDFormat)
	if err != nil {
		return nil, err
	}

	w.IDFromFile, err = makeIDFromFileFunc(cfg.Notes.IDFormat, w.IsValidNoteID)
	if err != nil {
		return nil, err
	}

	w.IDFromLink, err = makeIDFromLinkFunc(cfg.Notes.IDLinkFormat, w.IsValidNoteID)
	if err != nil {
		return nil, err
	}

	w.Homepage, err = parseHomepage(cfg.Homepage, w.IsValidNoteID)
	if err != nil {
		return nil, err
	}

	w.Tags, err = loadTags(cfg.Tags, w.IsValidNoteID)
	if err != nil {
		return nil, err
	}

	for _, m := range cfg.Menu {
		menu, err := parseMenu(m, w.IsValidNoteID, w.isTagPublished)
		if err != nil {
			return nil, err
		}
		w.Menu = append(w.Menu, menu)
	}

	w.Feed, err = parseFeed(cfg.Feed, "Feed", w.IsValidNoteID)
	if err != nil {
		return nil, err
	}

	w.Pages.Tag = dd.Tag(cfg.Pages.Tag)

	w.HeadSuffix = cfg.Segments.HeadSuffix
	w.NoteSuffix = cfg.Segments.NoteSuffix
	w.FooterPrefix = cfg.Segments.FooterPrefix

	w.ThemeCSS = themeCSS
	w.Favicon = faviconFile
	w.FaviconType = "image/svg+xml"

	return &w, nil
}

func readConfigFile(configPath string) (data.ConfigFile, error) {
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

func makeIDFromFileFunc(r string, isValid dd.NoteIDValidFunc) (dd.IDFromFileFunc, error) {
	// Load file ID regex from config and try to compile.
	validID, err := regexp.Compile(r)
	if err != nil {
		return nil, fmt.Errorf("could not compile regular expression '%s': %v", r, err)
	}

	return func(test string) (dd.NoteID, bool) {
		var id = validID.FindString(test)
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

func (w *WebsiteLang) TagsToPublished(t []dd.Tag) []Tag {
	tags := []Tag{}

	for _, tag := range t {
		if p, ok := w.Tags[tag]; ok {
			tags = append(tags, p)
		}
	}

	sort.Slice(tags, func(i, j int) bool {
		return tags[i].Title < tags[j].Title
	})

	return tags
}

func (w *WebsiteLang) Str(key l10n.Key) string {
	return w.localizer.Str(key)
}
