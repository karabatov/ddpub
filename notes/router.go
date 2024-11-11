package notes

import (
	"fmt"
	"html/template"
	"io"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/gorilla/feeds"
	"github.com/k3a/html2text"
	"github.com/karabatov/ddpub/config"
	"github.com/karabatov/ddpub/dd"
	"github.com/karabatov/ddpub/l10n"
	"github.com/karabatov/ddpub/layout"
)

type contentFunc func() (template.HTML, error)

type Router struct {
	website  *config.WebsiteLang
	routes   map[string]http.HandlerFunc
	pageWith func(pattern string, title string, content template.HTML) layout.Page
}

func newRouter(w *config.WebsiteLang, s *Store) (*Router, error) {
	r := Router{
		website: w,
		routes:  make(map[string]http.HandlerFunc),
		pageWith: func(pattern string, title string, content template.HTML) layout.Page {
			return layout.Page{
				Language: w.Language.String(),
				Head: layout.Head{
					Title:        title,
					WebsiteTitle: w.Title,
					MetaTags: layout.MetaTags{
						Title:    title,
						Type:     "website",
						Image:    template.HTML(w.AbsoluteURL(w.URLForSharedFile("og.jpg"))),
						URL:      template.HTML(w.AbsoluteURL(pattern)),
						Locale:   w.Language.Full(),
						SiteName: w.Title,
						Twitter:  w.Twitter,
					},
					Suffix: template.HTML(w.HeadSuffix),
				},
				Header: layout.Header{
					HomepageURL: template.HTML(w.URLForHomePage()),
					Title:       template.HTML(w.Title),
					Menu:        layoutMenu(w, s),
				},
				Content: content,
				Footer: layout.Footer{
					Prefix:    template.HTML(w.FooterPrefix),
					PoweredBy: template.HTML(w.Str(l10n.FooterPoweredBy)),
				},
			}
		},
	}

	// Add homepage.
	switch w.Homepage.Kind() {
	case config.HomepageKindNoteID:
		id := w.Homepage.(config.HomepageNoteID).ID
		note := s.noteContent[id]
		if err := r.addHandlerFor(w.URLForHomePage(), htmlAsText(note.title), func() (template.HTML, error) {
			return htmlForPage(&note, s)
		}); err != nil {
			return nil, err
		}
	case config.HomepageKindFeed:
		if err := r.addHandlerFor(w.URLForHomePage(), w.Feed.Title, func() (template.HTML, error) {
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
	if err := r.addHandlerFor(w.URLForBuiltin(dd.BuiltinTags), w.Str(l10n.TagsTitle), func() (template.HTML, error) {
		return htmlForBuiltinTags(w, s)
	}); err != nil {
		return nil, err
	}

	// Add published pages and notes.

	for _, p := range s.pub {
		note := s.noteContent[p.id]
		switch p.target {
		case publishTargetBuiltin, publishTargetTag:
			continue
		case publishTargetFeed:
			if err := r.addHandlerFor(w.URLForFeedNote(note.slug), htmlAsText(note.title), func() (template.HTML, error) {
				return htmlForNote(&note, w)
			}); err != nil {
				return nil, err
			}
		case publishTargetPage:
			if err := r.addHandlerFor(w.URLForPageNote(note.slug), htmlAsText(note.title), func() (template.HTML, error) {
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

	// Add shared files (only need to do this for main config).
	if !w.IsChild {
		for _, f := range w.SharedFiles {
			if err := r.addHandler(w.URLForSharedFile(f.Filename), handlerForFile(f.Content, f.ContentType)); err != nil {
				return nil, err
			}
		}
	}

	// Add files.

	for _, f := range s.files {
		if err := r.addHandler(f.link, handlerForLocalFile(f)); err != nil {
			return nil, err
		}
	}

	// Add RSS.

	rss := &feeds.Feed{
		Title:   w.Title,
		Link:    &feeds.Link{Href: w.AbsoluteURL(w.URLForHomePage())},
		Author:  &feeds.Author{},
		Updated: time.Now(),
	}
	for _, p := range s.pub {
		note := s.noteContent[p.id]
		if p.target != publishTargetFeed {
			continue
		}
		link := w.AbsoluteURL(w.URLForFeedNote(note.slug))
		rss.Add(&feeds.Item{
			Title:   htmlAsText(note.title),
			Id:      link,
			Link:    &feeds.Link{Href: link},
			Updated: note.updatedDate,
			Created: note.date,
			Content: string(note.content),
		})
	}
	rssFeed, err := rss.ToRss()
	if err != nil {
		return nil, err
	}
	if err := r.addHandler(w.URLForRSSFeed(), handlerForRSSFeed(rssFeed)); err != nil {
		return nil, err
	}

	return &r, nil
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
	h, err := handlerForPage(pattern, page)
	if err != nil {
		return err
	}

	return r.addHandler(pattern, h)
}

func (r *Router) addHandlerFor(pattern string, title string, content contentFunc) error {
	rendered, err := content()
	if err != nil {
		return err
	}

	page := r.pageWith(pattern, title, rendered)

	if err := r.addHandlerForPage(pattern, page); err != nil {
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

func handlerForRSSFeed(f string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/rss+xml")
		io.WriteString(w, f)
	}
}

func handlerForPage(pattern string, p layout.Page) (http.HandlerFunc, error) {
	l, err := layout.FillPage(p)
	if err != nil {
		return nil, err
	}

	return func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != pattern {
			http.NotFound(w, r)
			return
		}
		w.Write(l)
	}, nil
}

func layoutMenu(w *config.WebsiteLang, s *Store) []layout.ListItem {
	menu := []layout.ListItem{}
	for _, m := range w.Menu {
		var url string
		switch m := m.(type) {
		case config.MenuBuiltin:
			url = w.URLForBuiltin(m.Builtin)
		case config.MenuNoteID:
			url = w.URLForPageNote(s.noteContent[m.ID].slug)
		case config.MenuTag:
			url = w.URLForTag(w.Tags[m.Tag])
		case config.MenuURL:
			url = m.URL
		}
		menu = append(menu, layout.ListItem{
			Title: template.HTML(m.Title()),
			URL:   template.HTML(url),
		})
	}
	return menu
}

func htmlAsText(t template.HTML) string {
	return strings.TrimSpace(html2text.HTML2Text(string(t)))
}
