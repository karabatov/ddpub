package layout

// Embed filesystem for the `templates` directory.
// Preloads templates from the embedded files on start.
// Provides types to be slotted into the templates (mapped by caller).

import (
	"bytes"
	"embed"
	"html/template"
	"strings"
)

var (
	//go:embed templates/*
	files embed.FS
	tmpl  = template.Must(template.ParseFS(files, "templates/*.html"))
)

type Page struct {
	Language string
	Head     struct {
		Title       string
		ThemeCSSURL string
	}
	Header struct {
		Title    string
		Subtitle string
	}
	// Convert to []byte maybe
	Content string
	Menu    struct{}
	Footer  struct{}
}

type ContentPage struct {
	Title   string
	Content string
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
