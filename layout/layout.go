package layout

import (
	"bytes"
	"embed"
	"html/template"
	"strings"
)

var (
	//go:embed templates/*.html
	files embed.FS
	tmpl  = template.Must(template.ParseFS(files, "templates/*.html"))
)

type Head struct {
	Title       string
	ThemeCSSURL string
}

type Header struct {
	Title    string
	Subtitle string
}

type Page struct {
	Language string
	Head     Head
	Header   Header
	Content  template.HTML
	Menu     struct{}
	Footer   struct{}
}

type ContentPage struct {
	Title   string
	Content template.HTML
}

func FillPage(p Page) ([]byte, error) {
	var b bytes.Buffer

	if err := tmpl.ExecuteTemplate(&b, "base", p); err != nil {
		return nil, err
	}

	return b.Bytes(), nil
}

func FillContentPage(p ContentPage) (string, error) {
	s := new(strings.Builder)
	if err := tmpl.ExecuteTemplate(s, "content_page", p); err != nil {
		return "", err
	}

	return s.String(), nil
}
