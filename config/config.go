// Package config loads and validates website config from a directory.
package config

import (
	"fmt"

	"github.com/karabatov/ddpub/dd"
)

// Website represents the configuration of a website.
type Website struct {
	Homepage      Homepage
	Menu          []Menu
	IsValidNoteID dd.NoteIDValidFunc
}

func Load(configDir string) (Website, error) {
	return Website{}, fmt.Errorf("not implemented")
}
