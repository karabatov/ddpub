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

type Head struct {
	Title       string
	ThemeCSSURL template.HTML
}

type Header struct {
	Title    string
	Subtitle string
}

type Menu struct {
	Title string
	URL   template.HTML
}

type Page struct {
	Language string
	Head     Head
	Header   Header
	Content  template.HTML
	Menu     []Menu
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

func FillContentPage(p ContentPage) (template.HTML, error) {
	var b bytes.Buffer
	if err := tmpl.ExecuteTemplate(&b, "content_page", p); err != nil {
		return "", err
	}

	return template.HTML(b.Bytes()), nil
}
