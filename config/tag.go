package config

import (
	"fmt"

	"github.com/karabatov/ddpub/config/internal/data"
	"github.com/karabatov/ddpub/dd"
)

type Tag struct {
	Tag   dd.Tag
	ID    dd.NoteID
	Slug  string
	Title string
}

func parseTag(t data.Tag, isValid dd.NoteIDValidFunc) (Tag, error) {
	var tag Tag

	// At least the tag must be present.
	if len(t.Tag) == 0 {
		return Tag{}, fmt.Errorf("tag in [[tags]] cannot be empty")
	}
	tag.Tag = dd.Tag(t.Tag)

	// Default the slug to the tag itself.
	if len(t.Slug) == 0 {
		tag.Slug = t.Tag
	} else {
		tag.Slug = t.Slug
	}

	// Default the title to the mapped tag.
	if len(t.Title) == 0 {
		tag.Title = tag.Slug
	} else {
		tag.Title = t.Title
	}

	// If note ID is present, it must be valid.
	if len(t.ID) > 0 && !isValid(t.ID) {
		return Tag{}, fmt.Errorf("invalid note ID '%s' in tag '%s'", t.ID, t.Tag)
	}
	// At this point it's eithe empty or valid.
	tag.ID = dd.NoteID(t.ID)

	// Tag description is good.
	return tag, nil
}
