package config

import "github.com/karabatov/ddpub/dd"

type HomepageKind int

const (
	HomepageKindFeed HomepageKind = iota
	HomepageKindNoteID
)

type Homepage interface {
	Kind() HomepageKind
}

type HomepageFeed struct {}

func (h HomepageFeed) Kind() HomepageKind {
	return HomepageKindFeed
}

type HomepageNoteID struct {
	ID dd.NoteID
}

func (h HomepageNoteID) Kind() HomepageKind {
	return HomepageKindNoteID
}
