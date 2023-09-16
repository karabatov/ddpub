package config

import (
	"fmt"

	"github.com/karabatov/ddpub/config/internal/data"
	"github.com/karabatov/ddpub/dd"
)

type Feed struct {
	Tag       dd.Tag
	URLPrefix string
	ID        dd.NoteID
	Title     string
}

func parseFeed(f data.Feed, defaultTitle string, isValid dd.NoteIDValidFunc) (Feed, error) {
	var feed Feed

	if len(f.Tag) == 0 {
		return Feed{}, fmt.Errorf("feed tag cannot be empty")
	}
	feed.Tag = dd.Tag(f.Tag)

	// Don't forget to verify the URL prefix later when building the router.
	feed.URLPrefix = "feed"
	if len(f.URLPrefix) > 0 {
		feed.URLPrefix = f.URLPrefix
	}

	if len(f.ID) > 0 && !isValid(f.ID) {
		return Feed{}, fmt.Errorf("invalid note ID '%s' in feed", f.ID)
	}
	feed.ID = dd.NoteID(f.ID)

	feed.Title = defaultTitle
	if len(f.Title) > 0 {
		feed.Title = f.Title
	}

	return feed, nil
}
