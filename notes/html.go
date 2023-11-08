package notes

import (
	"html/template"
	"sort"

	"github.com/karabatov/ddpub/config"
	"github.com/karabatov/ddpub/dd"
	"github.com/karabatov/ddpub/l10n"
	"github.com/karabatov/ddpub/layout"
)

func htmlForPage(note *noteContent, s *Store) (template.HTML, error) {
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

func feedNotesListItems(t dd.Tag, w *config.Website, s *Store) []layout.NoteListItem {
	notes := []layout.NoteListItem{}
	for _, n := range s.notesForTag(t) {
		nli := layout.NoteListItem{
			ListItem: layout.ListItem{
				Title: n.title,
				URL:   template.HTML(w.URLForFeedNote(n.slug)),
			},
			Date: n.date.Format(w.Str(l10n.DateFormat)),
		}
		notes = append(notes, nli)
	}
	return notes
}

func tagsListItems(w *config.Website) []layout.ListItem {
	tags := []layout.ListItem{}
	for _, t := range w.Tags {
		li := layout.ListItem{
			Title: template.HTML(t.Title),
			URL:   template.HTML(w.URLForTag(t)),
		}
		tags = append(tags, li)
	}
	sort.Slice(tags, func(i, j int) bool {
		return tags[i].Title < tags[j].Title
	})
	return tags
}

func htmlForBuiltinFeed(w *config.Website, s *Store) (template.HTML, error) {
	var content template.HTML
	if len(w.Feed.ID) > 0 {
		note := s.noteContent[w.Feed.ID]
		content = template.HTML(note.content)
	}

	p := layout.BuiltinFeed{
		Title:   template.HTML(w.Feed.Title),
		Content: content,
		Notes:   feedNotesListItems(w.Feed.Tag, w, s),
	}
	rendered, err := layout.FillBuiltinFeed(p)
	if err != nil {
		return "", err
	}
	return rendered, nil
}

func htmlForBuiltinTags(w *config.Website) (template.HTML, error) {
	p := layout.BuiltinTags{
		Title: template.HTML(w.Str(l10n.TagsTitle)),
		Tags:  tagsListItems(w),
	}
	rendered, err := layout.FillBuiltinTags(p)
	if err != nil {
		return "", err
	}
	return rendered, nil
}

func htmlForTag(t *config.Tag, w *config.Website, s *Store) (template.HTML, error) {
	var content template.HTML
	if len(t.ID) > 0 {
		note := s.noteContent[t.ID]
		content = template.HTML(note.content)
	}

	tp := layout.ContentTagPage{
		Title:   template.HTML(t.Title),
		Content: content,
		Notes:   feedNotesListItems(t.Tag, w, s),
	}
	rendered, err := layout.FillContentTagPage(tp)
	if err != nil {
		return "", err
	}
	return rendered, nil
}

func htmlForNote(note *noteContent, w *config.Website) (template.HTML, error) {
	tags := []layout.ListItem{}
	for _, t := range w.TagsToPublished(note.tags) {
		tags = append(tags, layout.ListItem{
			Title: template.HTML(t.Title),
			URL:   template.HTML(w.URLForTag(t)),
		})
	}
	cn := layout.ContentNote{
		Title:   note.title,
		Date:    note.date.Format(w.Str(l10n.DateFormat)),
		Content: template.HTML(note.content),
		Tags:    tags,
		Suffix:  template.HTML(w.NoteSuffix),
	}
	rendered, err := layout.FillContentNote(cn)
	if err != nil {
		return "", err
	}
	return rendered, nil
}
