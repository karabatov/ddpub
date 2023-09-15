// Package dd contains core types and functions that are integral to DDPub as a whole.
package dd

// NoteID is a valid note ID.
type NoteID string

// NoteIDValidFunc is the function type to check if the note ID is valid.
type NoteIDValidFunc func(string) bool

// Tag represents a tag (no hashtag).
type Tag string

// Builtin enumerates built-in DDPub pages.
type Builtin int

const (
	BuiltinFeed = iota + 1
	BuiltinSearch
	BuiltinTags
)

func (b Builtin) IsValid() bool {
	return b >= BuiltinFeed && b <= BuiltinTags
}

// Language represents a supported language.
type Language string

const (
	LanguageEnUS = "en-US"
)
