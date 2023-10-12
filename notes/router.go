package notes

import (
	"fmt"
	"html/template"
	"log"
	"net/http"
	"os"

	"github.com/karabatov/ddpub/config"
	"github.com/karabatov/ddpub/dd"
	"github.com/karabatov/ddpub/layout"
)

type contentFunc func() (template.HTML, error)

type Router struct {
	routes   map[string]http.HandlerFunc
	pageWith func(title string, content template.HTML) layout.Page
}

func NewRouter(w *config.Website, s *Store) (*Router, error) {
	r := Router{routes: make(map[string]http.HandlerFunc)}

	menu := layoutMenu(w, s)
	r.pageWith = func(title string, content template.HTML) layout.Page {
		return layout.Page{
			Language: "en-US",
			Head: layout.Head{
				Title:       title,
				ThemeCSSURL: template.HTML(w.URLForThemeCSS()),
			},
			Header: layout.Header{
				Title: w.Title,
				Menu:  menu,
			},
			Content: content,
			Footer:  struct{}{},
		}
	}

	// Add theme.css.
	if err := r.addHandler(w.URLForThemeCSS(), handlerForFile(w.ThemeCSS, "text/css")); err != nil {
		return nil, err
	}

	// Add homepage.
	switch w.Homepage.Kind() {
	case config.HomepageKindNoteID:
		id := w.Homepage.(config.HomepageNoteID).ID
		note := s.pub[id]
		if err := r.addHandlerFor("/", note.title, func() (template.HTML, error) {
			return htmlForPage(&note, s)
		}); err != nil {
			return nil, err
		}
	case config.HomepageKindFeed:
		if err := r.addHandlerFor("/", w.Feed.Title, func() (template.HTML, error) {
			return htmlForBuiltinFeed(w, s)
		}); err != nil {
			return nil, err
		}
	}

	// Add builtin pages.

	// Builtin - feed.
	if err := r.addHandlerFor(w.URLForBuiltin(dd.BuiltinFeed), w.Feed.Title, func() (template.HTML, error) {
		return htmlForBuiltinFeed(w, s)
	}); err != nil {
		return nil, err
	}

	// Builtin - tags.
	if err := r.addHandlerFor(w.URLForBuiltin(dd.BuiltinTags), "Tags", func() (template.HTML, error) {
		return htmlForBuiltinTags(w)
	}); err != nil {
		return nil, err
	}

	// Add pages from the menu.

	for _, m := range w.Menu {
		switch m := m.(type) {
		case config.MenuNoteID:
			note := s.pub[m.ID]
			if err := r.addHandlerFor(w.URLForMenuNote(note.slug), note.title, func() (template.HTML, error) {
				return htmlForPage(&note, s)
			}); err != nil {
				return nil, err
			}
		}
	}

	// Add published tags.

	for _, t := range w.Tags {
		if err := r.addHandlerFor(w.URLForTag(t), t.Title, func() (template.HTML, error) {
			return htmlForTag(&t, w, s)
		}); err != nil {
			return nil, err
		}
	}

	// Add published pages from the feed (if there are any).

	for _, note := range s.notesForTag(w.Feed.Tag) {
		if err := r.addHandlerFor(w.URLForFeedNote(note.slug), note.title, func() (template.HTML, error) {
			return htmlForNote(&note, w)
		}); err != nil {
			return nil, err
		}
	}

	// Add files.
	for _, f := range s.files {
		if err := r.addHandler(f.link, handlerForLocalFile(f)); err != nil {
			return nil, err
		}
	}

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

func (r *Router) addHandlerFor(url string, title string, content contentFunc) error {
	rendered, err := content()
	if err != nil {
		return err
	}

	page := r.pageWith(title, rendered)

	if err := r.addHandlerForPage(url, page); err != nil {
		return err
	}

	return nil
}

func handlerForFile(f []byte, contentType string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", contentType)
		w.Write(f)
	}
}

func handlerForLocalFile(f file) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", f.contentType)
		if r, err := os.ReadFile(f.path); err == nil {
			w.Write(r)
		} else {
			w.Write([]byte{})
		}
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

func layoutMenu(w *config.Website, s *Store) []layout.ListItem {
	menu := []layout.ListItem{}
	for _, m := range w.Menu {
		var url string
		switch m := m.(type) {
		case config.MenuBuiltin:
			url = w.URLForBuiltin(m.Builtin)
		case config.MenuNoteID:
			url = w.URLForMenuNote(s.pub[m.ID].slug)
		case config.MenuTag:
			url = w.URLForTag(w.Tags[m.Tag])
		case config.MenuURL:
			url = m.URL
		}
		menu = append(menu, layout.ListItem{
			Title: m.Title(),
			URL:   template.HTML(url),
		})
	}
	return menu
}
