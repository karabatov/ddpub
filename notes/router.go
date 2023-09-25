package notes

import (
	"fmt"
	"html/template"
	"log"
	"net/http"

	"github.com/karabatov/ddpub/config"
	"github.com/karabatov/ddpub/layout"
)

type Router struct {
	routes map[string]http.HandlerFunc
}

func NewRouter(w *config.Website, s *Store) (*Router, error) {
	r := Router{routes: make(map[string]http.HandlerFunc)}

	pageWith := func(title, content string) layout.Page {
		return layout.Page{
			Language: "en-US",
			Head:     layout.Head{Title: title},
			Header:   layout.Header{},
			Content:  template.HTML(content),
			Menu:     struct{}{},
			Footer:   struct{}{},
		}
	}

	// Add theme.css.
	if err := r.addHandler(w.URLForThemeCSS(), handlerForFile(w.ThemeCSS)); err != nil {
		return nil, err
	}

	// Add homepage.
	home := pageWith("Home", "<p>Hello, World!</p>")
	if err := r.addHandlerForPage("/", home); err != nil {
		return nil, err
	}

	// Add builtin pages.
	// Add pages from the menu.
	// Add tags.
	// Add published pages.
	// Add files.

	return &r, nil
}

func (r Router) ServeMux() *http.ServeMux {
	defer func() {
		// We shouldn't panic here, but it's better than crashing.
		if err := recover(); err != nil {
			log.Fatalf("Could not create router: %v", err)
		}
	}()

	mux := http.NewServeMux()

	for pattern, handler := range r.routes {
		mux.HandleFunc(pattern, handler)
	}

	return mux
}

func (r *Router) hasPattern(p string) bool {
	_, ok := r.routes[p]

	return ok
}

func (r *Router) addHandler(pattern string, handler http.HandlerFunc) error {
	if r.hasPattern(pattern) {
		return fmt.Errorf("pattern '%s' already registered with router", pattern)
	}

	r.routes[pattern] = handler
	return nil
}

func (r *Router) addHandlerForPage(pattern string, page layout.Page) error {
	h, err := handlerForPage(page)
	if err != nil {
		return err
	}

	return r.addHandler(pattern, h)
}

// Add header for file type?
func handlerForFile(f []byte) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		w.Write(f)
	}
}

func handlerForPage(p layout.Page) (http.HandlerFunc, error) {
	l, err := layout.FillPage(p)
	if err != nil {
		return nil, err
	}

	return func(w http.ResponseWriter, r *http.Request) {
		w.Write(l)
	}, nil
}
