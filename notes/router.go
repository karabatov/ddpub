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

	menu := layoutMenu(w, s)
	pageWith := func(title string, content template.HTML) layout.Page {
		return layout.Page{
			Language: "en-US",
			Head: layout.Head{
				Title:       title,
				ThemeCSSURL: template.HTML(w.URLForThemeCSS()),
			},
			Header:  layout.Header{},
			Content: content,
			Menu:    menu,
			Footer:  struct{}{},
		}
	}

	// Add theme.css.
	if err := r.addHandler(w.URLForThemeCSS(), handlerForFile(w.ThemeCSS)); err != nil {
		return nil, err
	}

	// Add homepage.
	switch w.Homepage.Kind() {
	case config.HomepageKindNoteID:
		id := w.Homepage.(config.HomepageNoteID).ID
		note := s.pub[id]
		rendered, err := htmlForPage(&note, s)
		if err != nil {
			return nil, err
		}
		if err := r.addHandlerForPage("/", pageWith(note.title, rendered)); err != nil {
			return nil, err
		}
	case config.HomepageKindFeed:
		return nil, fmt.Errorf("homepage feed not supported")
	}

	// Add builtin pages.

	// Add pages from the menu.

	for _, m := range w.Menu {
		switch m := m.(type) {
		case config.MenuNoteID:
			note := s.pub[m.ID]
			rendered, err := htmlForPage(&note, s)
			if err != nil {
				return nil, err
			}
			url := w.URLForMenuNote(note.slug)
			page := pageWith(note.title, rendered)
			if err := r.addHandlerForPage(url, page); err != nil {
				return nil, err
			}
		}
	}

	// Add published tags.

	for _, t := range w.Tags {
		rendered, err := htmlForTag(&t, w, s)
		if err != nil {
			return nil, err
		}
		url := w.URLForTag(t)
		page := pageWith(t.Title, rendered)
		if err := r.addHandlerForPage(url, page); err != nil {
			return nil, err
		}
	}

	// Add published pages from the feed (if there are any).

	for _, note := range s.notesForTag(w.Feed.Tag) {
		rendered, err := htmlForNote(&note, w)
		if err != nil {
			return nil, err
		}
		url := w.URLForFeedNote(note.slug)
		page := pageWith(note.title, rendered)
		if err := r.addHandlerForPage(url, page); err != nil {
			return nil, err
		}
	}

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

func htmlForPage(note *note, s *Store) (template.HTML, error) {
	cp := layout.ContentPage{
		Title:   note.title,
		Content: template.HTML(note.content),
	}
	rendered, err := layout.FillContentPage(cp)
	if err != nil {
		return "", err
	}
	return rendered, nil
}

func htmlForTag(t *config.Tag, w *config.Website, s *Store) (template.HTML, error) {
	var tagContent template.HTML
	if len(t.ID) > 0 {
		note := s.pub[t.ID]
		tagContent = template.HTML(note.content)
	}

	notes := []layout.NoteListItem{}
	for _, n := range s.notesForTag(t.Tag) {
		nli := layout.NoteListItem{
			ListItem: layout.ListItem{
				Title: n.title,
				URL:   template.HTML(w.URLForFeedNote(n.slug)),
			},
			Date: n.date,
		}
		notes = append(notes, nli)
	}

	tp := layout.ContentTagPage{
		Title:   t.Title,
		Content: tagContent,
		Notes:   notes,
	}
	rendered, err := layout.FillContentTagPage(tp)
	if err != nil {
		return "", err
	}
	return rendered, nil
}

func htmlForNote(note *note, w *config.Website) (template.HTML, error) {
	tags := []layout.ListItem{}
	for _, t := range w.TagsToPublished(note.tags) {
		tags = append(tags, layout.ListItem{
			Title: t.Title,
			URL:   template.HTML(w.URLForTag(t)),
		})
	}
	cn := layout.ContentNote{
		Title:   note.title,
		Content: template.HTML(note.content),
		Tags:    tags,
	}
	rendered, err := layout.FillContentNote(cn)
	if err != nil {
		return "", err
	}
	return rendered, nil
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
