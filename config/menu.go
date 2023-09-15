package config

import (
	"fmt"

	"github.com/karabatov/ddpub/config/internal/data"
	"github.com/karabatov/ddpub/dd"
)

// MenuKind represents the type of the menu entry.
type MenuKind int

const (
	MenuKindBuiltin = iota + 1
	MenuKindNoteID
	MenuKindTag
	MenuKindURL
)

type Menu interface {
	Kind() MenuKind
	Title() string
}

type menuEntry struct {
	kind  MenuKind
	title string
}

func (m menuEntry) Kind() MenuKind {
	return m.kind
}

func (m menuEntry) Title() string {
	return m.title
}

type MenuBuiltin struct {
	menuEntry

	Builtin dd.Builtin
}

type MenuNoteID struct {
	menuEntry

	ID dd.NoteID
}

type MenuTag struct {
	menuEntry

	Tag dd.Tag
}

type MenuURL struct {
	menuEntry

	URL string
}

func validate(m data.Menu) error {
	// Check that only one field is filled
	filled := 0
	if len(m.Builtin) > 0 {
		filled += 1
	}
	if len(m.ID) > 0 {
		filled += 1
	}
	if len(m.Tag) > 0 {
		filled += 1
	}
	if len(m.URL) > 0 {
		filled += 1
	}

	if filled != 1 {
		return fmt.Errorf("menu entry can only have one type")
	}

	return nil
}

func parse(m data.Menu, isValidID func(string) bool) (Menu, error) {
	if err := validate(m); err != nil {
		return menuEntry{}, err
	}

	if len(m.Builtin) > 0 {
		return parseMenuBuiltin(m)
	}

	if len(m.ID) > 0 {
		return parseMenuNoteID(m, isValidID)
	}

	if len(m.Tag) > 0 {
		return parseMenuTag(m)
	}

	if len(m.URL) > 0 {
		return parseMenuURL(m)
	}

	panic("unreachable")
}

func parseMenuBuiltin(m data.Menu) (Menu, error) {
	var menu MenuBuiltin
	menu.kind = MenuKindBuiltin
	menu.title = m.Title
	switch m.Builtin {
	case "feed":
		menu.Builtin = dd.BuiltinFeed
	case "search":
		menu.Builtin = dd.BuiltinSearch
	case "tags":
		menu.Builtin = dd.BuiltinTags
	default:
		return menuEntry{}, fmt.Errorf("unknown builtin '%s'", m.Builtin)
	}
	return menu, nil
}

func parseMenuNoteID(m data.Menu, isValidID func(string) bool) (Menu, error) {
	var menu MenuNoteID
	menu.kind = MenuKindNoteID
	menu.title = m.Title
	if !isValidID(m.ID) {
		return menuEntry{}, fmt.Errorf("invalid note id '%s'", m.ID)
	}
	menu.ID = m.ID
	return menu, nil
}

func parseMenuTag(m data.Menu) (Menu, error) {
	var menu MenuTag
	menu.kind = MenuKindTag
	menu.title = m.Title
	menu.Tag = m.Tag
	return menu, nil
}

func parseMenuURL(m data.Menu) (Menu, error) {
	var menu MenuURL
	menu.kind = MenuKindURL
	menu.title = m.Title
	menu.URL = m.URL
	return menu, nil
}
