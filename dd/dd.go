// Package dd contains core types and functions that are integral to DDPub as a whole.
package dd

import "regexp"

// NoteID is a valid note ID.
type NoteID string

// NoteIDValidFunc is the function type to check if the note ID is valid.
type NoteIDValidFunc func(string) bool

// IDFromLinkFunc is the function type to extract the note ID from a markdown link.
// It returns the extracted note ID and a boolean indicating whether the ID is valid.
type IDFromLinkFunc func(string) (NoteID, bool)

// IDFromFileFunc is the function type to extract the note ID from a filename.
// It returns the extracted note ID and a boolean indicating whether the ID is valid.
type IDFromFileFunc func(string) (NoteID, bool)

// Tag represents a tag (no hashtag).
type Tag string

// Builtin enumerates built-in DDPub pages.
type Builtin int

const (
	BuiltinFeed Builtin = iota + 1
	BuiltinSearch
	BuiltinTags
)

func (b Builtin) IsValid() bool {
	return b >= BuiltinFeed && b <= BuiltinTags
}

// Language represents a supported language.
type Language string

const (
	LanguageEnUS Language = "en-US"
)

func FirstSubmatch(re *regexp.Regexp, line string) (string, bool) {
	if matches := re.FindStringSubmatch(line); len(matches) > 1 {
		return matches[1], true
	}

	return "", false
}
