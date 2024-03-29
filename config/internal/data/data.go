// Package data contains type definitions for TOML unmarshalling.
package data

type Homepage struct {
	ID string `toml:"id"`
}

type Language struct {
	Code     string
	UseShort bool `toml:"short"`
}

type Feed struct {
	Tag       string
	URLPrefix string `toml:"url_prefix"`
	ID        string `toml:"id"`
	Title     string
}

type Pages struct {
	Tag string
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

type Segments struct {
	HeadSuffix   string `toml:"head_suffix"`
	NoteSuffix   string `toml:"note_suffix"`
	FooterPrefix string `toml:"footer_prefix"`
}

// ConfigFile represents a TOML configuration file for a single website.
type ConfigFile struct {
	Domain   string
	HTTPS    bool
	Twitter  string
	Title    string
	Language Language
	Feed     Feed
	Pages    Pages
	Homepage Homepage
	Menu     []Menu
	Notes    struct {
		IDFormat     string `toml:"id_format"`
		IDLinkFormat string `toml:"id_link_format"`
	}
	Tags     []Tag
	Segments Segments
}
