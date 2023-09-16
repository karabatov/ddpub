package config

import (
	"fmt"

	"github.com/karabatov/ddpub/config/internal/data"
	"github.com/karabatov/ddpub/dd"
)

type HomepageKind int

const (
	HomepageKindFeed HomepageKind = iota
	HomepageKindNoteID
)

type Homepage interface {
	Kind() HomepageKind
}

type HomepageFeed struct{}

func (h HomepageFeed) Kind() HomepageKind {
	return HomepageKindFeed
}

type HomepageNoteID struct {
	ID dd.NoteID
}

func (h HomepageNoteID) Kind() HomepageKind {
	return HomepageKindNoteID
}

func parseHomepage(h data.Homepage, isValid dd.NoteIDValidFunc) (Homepage, error) {
	if len(h.ID) > 0 {
		if isValid(h.ID) {
			return HomepageNoteID{dd.NoteID(h.ID)}, nil
		}

		return nil, fmt.Errorf("invalid note id '%s'", h.ID)
	}

	return HomepageFeed{}, nil
}
