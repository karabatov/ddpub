// Package notes is responsible for loading notes from the notes directory.
package notes

import (
	"regexp"
	"time"

	"github.com/gomarkdown/markdown/ast"
	"github.com/karabatov/ddpub/dd"
)

var (
	matchMarkdownFile = regexp.MustCompile(`.md$`)
	// Matches title line.
	matchLineTitle = regexp.MustCompile(`^#\s(.*)$`)
	// Matches the line with the date.
	matchLineDate = regexp.MustCompile(`^Date:\s(.*)\s*$`)
	// Matches the line with the language.
	matchLineLanguage = regexp.MustCompile(`^Language:\s(.*)\s*$`)
	// Matches the line with the slug.
	matchLineSlug = regexp.MustCompile(`^Slug:\s(.*)\s*$`)
	// Matches the line with tags.
	matchLineTags = regexp.MustCompile(`^Tags:\s(.*)\s*$`)
	// Matches one tag without the pound sign.
	matchOneTag = regexp.MustCompile(`#(\S+)\s*`)
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
