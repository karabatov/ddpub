package layout

import (
	"bytes"
	"embed"
	"html/template"
)

var (
	//go:embed templates/*.html
	files embed.FS
	tmpl  = template.Must(template.ParseFS(files, "templates/*.html"))
)

type MetaTags struct {
	Title    string
	Type     string
	Image    template.HTML
	URL      template.HTML
	Locale   string
	SiteName string
	Twitter  string
}

type Head struct {
	Title        string
	WebsiteTitle string
	MetaTags     MetaTags
	RSSURL       template.HTML
	Suffix       template.HTML
}

type Header struct {
	HomepageURL template.HTML
	Title       template.HTML
	Menu        []ListItem
}

type Footer struct {
	Prefix    template.HTML
	PoweredBy template.HTML
	RSSURL    template.HTML
}

// ListItem represents an item in the list with a link and title.
type ListItem struct {
	Title template.HTML
	URL   template.HTML
}

type Page struct {
	Language string
	Head     Head
	Header   Header
	Content  template.HTML
	Footer   Footer
}

// BuiltinFeed contains data to render the content of the builtin feed page.
type BuiltinFeed struct {
	Title   template.HTML
	Content template.HTML
	Notes   []NoteListItem
}

// TagListItem represents the tag in a list of tags. Count is the number of
// posts on the feed for a particular tag.
type TagListItem struct {
	ListItem

	Count int
}

// BuiltinTags contains data to render the content of the builtin tags page.
type BuiltinTags struct {
	Title template.HTML
	Tags  []TagListItem
}

type ContentPage struct {
	Title   template.HTML
	Content template.HTML
}

// NoteListItem represents a note in the list on a page: feed or tag.
type NoteListItem struct {
	ListItem

	Date string
}

type ContentTagPage struct {
	Title   template.HTML
	Content template.HTML
	Notes   []NoteListItem
}

type ContentNote struct {
	Title   template.HTML
	Date    string
	Tags    []ListItem
	Content template.HTML
	Suffix  template.HTML
}

func FillPage(p Page) ([]byte, error) {
	var b bytes.Buffer

	if err := tmpl.ExecuteTemplate(&b, "base", p); err != nil {
		return nil, err
	}

	return b.Bytes(), nil
}

type content interface {
	BuiltinFeed | BuiltinTags | ContentPage | ContentTagPage | ContentNote
}

func fillContent[C content](content C, tName string) (template.HTML, error) {
	var b bytes.Buffer
	if err := tmpl.ExecuteTemplate(&b, tName, content); err != nil {
		return "", err
	}

	return template.HTML(b.Bytes()), nil
}

func FillBuiltinFeed(p BuiltinFeed) (template.HTML, error) {
	return fillContent(p, "builtin_feed")
}

func FillBuiltinTags(p BuiltinTags) (template.HTML, error) {
	return fillContent(p, "builtin_tags")
}

func FillContentPage(p ContentPage) (template.HTML, error) {
	return fillContent(p, "content_page")
}

func FillContentTagPage(p ContentTagPage) (template.HTML, error) {
	return fillContent(p, "content_tag")
}

func FillContentNote(p ContentNote) (template.HTML, error) {
	return fillContent(p, "content_note")
}
