// Package dd contains core types and functions that are integral to DDPub as a whole.
package dd

// NoteID is a valid note ID.
type NoteID = string

// Tag represents a tag (no hashtag).
type Tag = string

// Builtin enumerates built-in DDPub pages.
type Builtin = int

const (
	BuiltinFeed = iota + 1
	BuiltinSearch
	BuiltinTags
)

// Language represents a supported language.
type Language = string

const (
	LanguageEnUS = "en-US"
)
