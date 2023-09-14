// Package config loads and validates website config from a directory.
package config

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
