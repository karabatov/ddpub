// Package config loads and validates website config from a directory.
package config

// MenuEntry always has a title and can be either of:
//   - builtin
//   - named note with id
//   - tag
//   - url
type MenuEntry struct {
	Title   string
	Builtin string
	ID      string `toml:"id"`
	Tag     string
	URL     string `toml:"url"`
}

const (
	MenuInvalid = iota
	MenuBuiltinFeed
	MenuBuiltinSearch
	MenuBuiltinTags
	MenuNoteID
	MenuTag
	MenuURL
)

// Returns one of the Menu* const values.
func (m MenuEntry) kind() int {
	kind := MenuInvalid

	if len(m.Builtin) > 0 {
		switch m.Builtin {
		case "feed":
			kind = MenuBuiltinFeed
		case "search":
			kind = MenuBuiltinSearch
		case "tags":
			kind = MenuBuiltinTags
		default:
			return MenuInvalid
		}
	}

	// Can't verify if the note ID is valid or not here.
	if len(m.ID) > 0 {
		if kind == MenuInvalid {
			kind = MenuNoteID
		} else {
			return MenuInvalid
		}
	}

	if len(m.Tag) > 0 {
		if kind == MenuInvalid {
			kind = MenuTag
		} else {
			return MenuInvalid
		}
	}

	if len(m.URL) > 0 {
		if kind == MenuInvalid {
			kind = MenuURL
		} else {
			return MenuInvalid
		}
	}

	return kind
}

type Homepage struct {
	ID string `toml:"id"`
}

type HomepageKind int

const (
	HomepageFeed HomepageKind = iota
	HomepageNoteID
)

func (h Homepage) kind() HomepageKind {
	if len(h.ID) > 0 {
		return HomepageNoteID
	}

	return HomepageFeed
}

// DDConfig represents a configuration file for a single website.
type DDConfig struct {
	Address string
	Feed    struct {
		Tag string
	}
	Homepage struct {
		Id string `toml:"id"`
	}
	Menu []struct {
		Title   string
		Builtin string
		ID      string `toml:"id"`
		Tag     string
		URL     string `toml:"url"`
	}
	Notes struct {
		IdFormat     string `toml:"id_format"`
		IdLinkFormat string `toml:"id_link_format"`
	}
	Tags []struct {
		Tag   string
		ID    string `toml:"id"`
		Slug  string
		Title string
	}
}
