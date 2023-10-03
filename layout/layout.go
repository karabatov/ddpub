package layout

import (
	"bytes"
	"embed"
	"html/template"
	"time"
)

var (
	//go:embed templates/*.html
	files embed.FS
	tmpl  = template.Must(template.ParseFS(files, "templates/*.html"))
)

type Head struct {
	Title       string
	ThemeCSSURL template.HTML
}

type Header struct {
	Title    string
	Subtitle string
}

// ListItem represents an item in the list with a link and title.
type ListItem struct {
	Title string
	URL   template.HTML
}

type Page struct {
	Language string
	Head     Head
	Header   Header
	Content  template.HTML
	Menu     []ListItem
	Footer   struct{}
}

type ContentPage struct {
	Title   string
	Content template.HTML
}

// NoteListItem represents a note in the list on a page: feed or tag.
type NoteListItem struct {
	ListItem

	Date time.Time
}

type ContentTagPage struct {
	Title   string
	Content template.HTML
	Notes   []NoteListItem
}

type ContentNote struct {
	Title   string
	Tags    []ListItem
	Content template.HTML
}

func FillPage(p Page) ([]byte, error) {
	var b bytes.Buffer

	if err := tmpl.ExecuteTemplate(&b, "base", p); err != nil {
		return nil, err
	}

	return b.Bytes(), nil
}

type content interface {
	ContentPage | ContentTagPage | ContentNote
}

func fillContent[C content](content C, tName string) (template.HTML, error) {
	var b bytes.Buffer
	if err := tmpl.ExecuteTemplate(&b, tName, content); err != nil {
		return "", err
	}

	return template.HTML(b.Bytes()), nil
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
