// Package data contains type definitions for TOML unmarshalling.
package data

type Homepage struct {
	Id string `toml:"id"`
}

type Menu struct {
	Title   string
	Builtin string
	ID      string `toml:"id"`
	Tag     string
	URL     string `toml:"url"`
}

type Tag struct {
	Tag   string
	ID    string `toml:"id"`
	Slug  string
	Title string
}

// ConfigFile represents a configuration file for a single website.
type ConfigFile struct {
	Address string
	Feed    struct {
		Tag string
	}
	Homepage Homepage
	Menu     []Menu
	Notes    struct {
		IdFormat     string `toml:"id_format"`
		IdLinkFormat string `toml:"id_link_format"`
	}
	Tags []Tag
}
