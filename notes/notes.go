// Package notes is responsible for loading notes from the notes directory.
package notes

import (
	"time"

	"github.com/gomarkdown/markdown/ast"
	"github.com/karabatov/ddpub/dd"
)

type Metadata struct {
	filename string
	modTime  time.Time
	date     time.Time
	title    string
	slug     string
	tags     []dd.Tag
	language string
}

type Note struct {
	meta Metadata
	doc  ast.Node
}
