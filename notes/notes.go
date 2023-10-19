// Package notes is responsible for loading notes from the notes directory.
package notes

import (
	"bufio"
	"fmt"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"time"

	"github.com/gomarkdown/markdown"
	"github.com/gomarkdown/markdown/ast"
	"github.com/gomarkdown/markdown/html"
	"github.com/gomarkdown/markdown/parser"
	"github.com/karabatov/ddpub/config"
	"github.com/karabatov/ddpub/dd"
)

var (
	matchMarkdownFile = regexp.MustCompile(`.md$`)
	// Matches title line.
	matchLineTitle = regexp.MustCompile(`^#\s(.*)$`)
	// Matches the line with the date.
	matchLineDate = regexp.MustCompile(`^Date:\s(.*)\s*$`)
	// Matches the line with the language.
	matchLineLanguage = regexp.MustCompile(`^Language:\s(.*)\s*$`)
	// Matches the line with the slug.
	matchLineSlug = regexp.MustCompile(`^Slug:\s(.*)\s*$`)
	// Matches the line with tags.
	matchLineTags = regexp.MustCompile(`^Tags:\s(.*)\s*$`)
	// Matches one tag without the pound sign.
	matchOneTag = regexp.MustCompile(`#(\S+)\s*`)
)

type metadata struct {
	id       dd.NoteID
	filename string
	modTime  time.Time
	date     time.Time
	title    string
	slug     string
	tags     []dd.Tag
	language string
}

type publishTarget int

const (
	publishTargetBuiltin publishTarget = iota + 1
	publishTargetFeed
	publishTargetPage
	publishTargetTag
)

type publishedNote struct {
	id     dd.NoteID
	target publishTarget
}

type noteContent struct {
	metadata

	doc     ast.Node
	content []byte
}

type file struct {
	// Link in the note.
	link string
	// Path on the file system.
	path string
	// Content type.
	contentType string
}

// Store captures the data necessary to publish the notes.
type Store struct {
	// Checks if the note ID is valid and metadata exists for it.
	NoteExists dd.NoteIDValidFunc
	// Metadata for all the notes in the notes directory.
	meta map[dd.NoteID]metadata
	// Notes grouped by tag.
	byTag map[dd.Tag][]dd.NoteID
	// Published notes.
	pub []publishedNote
	// Published notes' content.
	noteContent map[dd.NoteID]noteContent
	// Files found while scanning the note contents.
	files map[string]file
}

func NewStore(w *config.Website, notesDir string) (*Store, error) {
	var s Store
	var err error

	s.meta, err = readAllMetadata(notesDir, w.IDFromFile)
	if err != nil {
		return nil, err
	}

	s.NoteExists = func(test string) bool {
		if !w.IsValidNoteID(test) {
			return false
		}
		_, ok := s.meta[dd.NoteID(test)]
		return ok
	}

	s.byTag = makeNotesByTag(s.meta)

	s.files = make(map[string]file)

	s.pub = notesForExport(w, s.byTag)

	if err := s.readExportedContent(w, notesDir); err != nil {
		return nil, err
	}

	// Check that menu notes exist.
	for _, m := range w.Menu {
		if m, ok := m.(config.MenuNoteID); ok {
			if !s.isPageNote(w, m.ID) {
				return nil, fmt.Errorf("menu note not published: %s", m.ID)
			}
		}
	}

	return &s, nil
}

func readAllMetadata(notesDir string, idFromFile dd.IDFromFileFunc) (map[dd.NoteID]metadata, error) {
	// Read a list of “.md” files from the notes directory with names that match the regex.
	allFiles, err := os.ReadDir(notesDir)
	if err != nil {
		return nil, fmt.Errorf("could not read the notes directory '%s': %v", notesDir, err)
	}

	meta := make(map[dd.NoteID]metadata)
	for _, f := range allFiles {
		if f.IsDir() {
			continue
		}

		var filename = f.Name()
		id, ok := idFromFile(filename)
		if !ok || !matchMarkdownFile.MatchString(filename) {
			continue
		}

		fileMetadata, err := readMetadata(id, filename, notesDir)
		if err != nil {
			fmt.Println("Could not read metadata from file:", filename)
			continue
		}

		meta[id] = fileMetadata
	}

	return meta, nil
}

// Create a full list of unique tags (case-sensitive) present in the posts.
// Create a map of tag to list of file IDs with that tag.
func makeNotesByTag(m map[dd.NoteID]metadata) map[dd.Tag][]dd.NoteID {
	byTag := map[dd.Tag][]dd.NoteID{}

	for id, data := range m {
		for _, t := range data.tags {
			if tags, ok := byTag[t]; ok {
				byTag[t] = append(tags, id)
			} else {
				byTag[t] = []dd.NoteID{id}
			}
		}
	}

	return byTag
}

// Read metadata for the files in the list:
//   - Filename with extension (to be able to read it)
//   - File modification date
//   - Title (if present, defaults to blank)
//   - List of tags (if present, defaults to empty), with hashtag characters stripped
//   - Slug (if present, defaults to file ID)
//   - Date (if present, defaults to file modification date)
//   - Language (if present, defaults to default language code, currently "en-US")
//
// Metadata is read until the first line that _isn't_ metadata, so it all must be at the beginning of the file.
func readMetadata(id dd.NoteID, filename, directory string) (metadata, error) {
	var path = filepath.Join(directory, filename)
	data := metadata{id: id}

	file, err := os.Open(path)
	if err != nil {
		return data, err
	}
	defer file.Close()

	data.filename = filename
	data.modTime = fileModTime(file)

	s := bufio.NewScanner(file)
	for s.Scan() {
		if title, ok := dd.FirstSubmatch(matchLineTitle, s.Text()); ok {
			data.title = title
			continue
		}

		if tags, ok := dd.FirstSubmatch(matchLineTags, s.Text()); ok {
			data.tags = tagsFromLine(tags)
			continue
		}

		if slug, ok := dd.FirstSubmatch(matchLineSlug, s.Text()); ok {
			if len(slug) > 0 {
				data.slug = slug
			} else {
				data.slug = string(id)
			}
			continue
		}

		if lang, ok := dd.FirstSubmatch(matchLineLanguage, s.Text()); ok {
			data.language = lang
			continue
		}

		if date, ok := dd.FirstSubmatch(matchLineDate, s.Text()); ok {
			data.date, err = time.Parse(time.DateOnly, date)
			if err != nil {
				data.date = data.modTime
			}
			continue
		}

		// If no matchers match, we are done.
		break
	}

	// Set slug to id if no slug has been set.
	if len(data.slug) == 0 {
		data.slug = string(id)
	}

	return data, nil
}

func fileModTime(file *os.File) time.Time {
	if stat, err := file.Stat(); err == nil {
		return stat.ModTime()
	} else {
		return time.Now()
	}
}

func tagsFromLine(line string) []dd.Tag {
	tags := []dd.Tag{}
	for _, tagPair := range matchOneTag.FindAllStringSubmatch(line, -1) {
		tags = append(tags, dd.Tag(tagPair[1]))
	}
	return tags
}

// Build the complete list of *known* note IDs to be published before parsing).
// They are all valid, verified and exist in `notes`.
func notesForExport(w *config.Website, byTag map[dd.Tag][]dd.NoteID) []publishedNote {
	e := []publishedNote{}

	// Add the homepage note ID if it's there.
	if h, ok := w.Homepage.(config.HomepageNoteID); ok {
		e = append(e, publishedNote{
			id:     h.ID,
			target: publishTargetBuiltin,
		})
	}

	// Add the feed's note ID if it's there.
	if len(w.Feed.ID) > 0 {
		e = append(e, publishedNote{
			id:     w.Feed.ID,
			target: publishTargetBuiltin,
		})
	}

	// Add all named notes from [[tags]] to the list.
	for _, t := range w.Tags {
		if len(t.ID) > 0 {
			e = append(e, publishedNote{
				id:     t.ID,
				target: publishTargetTag,
			})
		}
	}

	// Add all notes with a publish tag from [pages] if it's there.
	if len(w.Pages.Tag) > 0 {
		for _, id := range byTag[w.Pages.Tag] {
			e = append(e, publishedNote{
				id:     id,
				target: publishTargetPage,
			})
		}
	}

	// Add all notes with a publish tag from [feed] if it's there.
	if len(w.Feed.Tag) > 0 {
		for _, id := range byTag[w.Feed.Tag] {
			e = append(e, publishedNote{
				id:     id,
				target: publishTargetFeed,
			})
		}
	}

	return e
}

func readContent(filename, directory string) ([]byte, error) {
	var path = filepath.Join(directory, filename)
	content := []byte{}

	file, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	s := bufio.NewScanner(file)
	appendLine := false
	for s.Scan() {
		if !appendLine {
			appendLine = appendLine || len(s.Bytes()) == 0
			continue
		}

		content = append(content, s.Bytes()...)
		content = append(content, byte('\n'))
	}

	return content, nil
}

// AST modification: https://github.com/gomarkdown/markdown/blob/master/examples/modify_ast.go
func modifyContent(noteAst ast.Node, modifyLink func(*ast.Link), modifyImage func(*ast.Image)) {
	ast.WalkFunc(noteAst, func(node ast.Node, entering bool) ast.WalkStatus {
		if !entering {
			return ast.GoToNext
		}

		switch typed := node.(type) {
		case *ast.Link:
			modifyLink(typed)
		case *ast.Image:
			modifyImage(typed)
		}
		return ast.GoToNext
	})
}

// Load up the notes' content. Convention: note content is considered
// to start after the first blank line. So content is everything between
// the first blank line and EOF.
func (s *Store) readExportedContent(w *config.Website, notesDir string) error {
	p := map[dd.NoteID]noteContent{}

	// Set up markdown parser.
	parserExtensions := parser.Tables | parser.FencedCode | parser.Strikethrough

	// Load note content.
	for _, pubNote := range s.pub {
		meta := s.meta[pubNote.id]

		// Skip if the content has already been read.
		if _, ok := p[pubNote.id]; ok {
			continue
		}

		content, err := readContent(meta.filename, notesDir)
		if err != nil {
			return fmt.Errorf("failed to load note with ID '%s': %v", pubNote.id, err)
		}

		// Parse note content with markdown parser.
		// https://github.com/gomarkdown/markdown/issues/280
		mp := parser.NewWithExtensions(parserExtensions)
		noteAst := mp.Parse(content)

		// Modify the AST:
		//  - Replace note links with URLs.
		//  - Complain and quit if any linked notes are not published.
		//  - Collect any links out to files (distinguish .md links from files?).
		modifyContent(noteAst, func(link *ast.Link) {
			linkStr := string(link.Destination)
			u, err := url.Parse(linkStr)
			if err != nil {
				return
			}
			// Might be a note link.
			id, ok := w.IDFromLink(u.Path)
			if !ok {
				// Some weird link, continue.
				return
			}

			if linkedMeta, ok := s.meta[id]; ok {
				newLink := linkStr
				if s.isFeedNote(w, id) {
					newLink = w.URLForFeedNote(linkedMeta.slug)
				} else if s.isPageNote(w, id) {
					newLink = w.URLForPageNote(linkedMeta.slug)
				}
				// Identifying tags by note ids is guessing so we don't do it.
				link.Destination = []byte(newLink)
			}

			// Continue if the link is external.
			if u.IsAbs() {
				link.AdditionalAttributes = append(link.AdditionalAttributes, `target="_blank"`)
				return
			}

			// Here we only care if the link is a file.
			if newFile, err := tryFileFromLink(linkStr, notesDir, w); err == nil {
				s.files[newFile.link] = newFile
				link.Destination = []byte(newFile.link)
			}
		}, func(image *ast.Image) {
			linkStr := string(image.Destination)
			if newFile, err := tryFileFromLink(linkStr, notesDir, w); err == nil {
				s.files[newFile.link] = newFile
				image.Destination = []byte(newFile.link)
			}
		})

		htmlFlags := html.CommonFlags | html.HrefTargetBlank
		opts := html.RendererOptions{Flags: htmlFlags}
		renderer := html.NewRenderer(opts)
		contentRendered := markdown.Render(noteAst, renderer)

		p[pubNote.id] = noteContent{metadata: meta, doc: noteAst, content: contentRendered}
	}

	s.noteContent = p
	return nil
}

func (s *Store) notesForTag(t dd.Tag) []noteContent {
	n := []noteContent{}

	for _, id := range s.byTag[t] {
		if p, ok := s.noteContent[id]; ok {
			n = append(n, p)
		}
	}

	sort.Slice(n, func(i, j int) bool {
		return n[i].date.After(n[j].date)
	})

	return n
}

func (s *Store) isFeedNote(w *config.Website, id dd.NoteID) bool {
	if len(w.Feed.Tag) == 0 {
		return false
	}

	for _, t := range s.meta[id].tags {
		if t == w.Feed.Tag {
			return true
		}
	}

	return false
}

func (s *Store) isPageNote(w *config.Website, id dd.NoteID) bool {
	if len(w.Pages.Tag) == 0 {
		return false
	}

	for _, t := range s.meta[id].tags {
		if t == w.Pages.Tag {
			return true
		}
	}

	return false
}

func fileContentType(f *os.File) string {
	buffer := make([]byte, 512)

	_, err := f.Read(buffer)
	if err != nil {
		return "application/octet-stream"
	}

	return http.DetectContentType(buffer)
}

func tryFileFromLink(link string, notesDir string, w *config.Website) (file, error) {
	u, err := url.Parse(link)
	if err != nil {
		return file{}, err
	}

	// Not a local file if it's an absolute URL.
	if u.IsAbs() {
		return file{}, fmt.Errorf("not a file link")
	}

	path := filepath.Join(notesDir, u.Path)
	f, err := os.Open(path)
	if err != nil {
		return file{}, err
	}
	defer f.Close()

	newLink := w.URLForFile(path)
	return file{
		link:        newLink,
		path:        path,
		contentType: fileContentType(f),
	}, nil
}
