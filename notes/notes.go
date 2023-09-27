// Package notes is responsible for loading notes from the notes directory.
package notes

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
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
	language dd.Language
}

type note struct {
	metadata

	doc     ast.Node
	content []byte
}

// Link to a file.
type link string

type file struct {
	// Link in the note.
	link link
	// Path on the file system.
	path string
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
	pub map[dd.NoteID]note
	// Files found while scanning the note contents.
	files map[link]file
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

	if err := s.readExportedContent(w, notesDir); err != nil {
		return nil, err
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
			data.language = dd.Language(lang)
			continue
		}

		if _, ok := dd.FirstSubmatch(matchLineDate, s.Text()); ok {
			// TODO: Define date format and parse.
			continue
		}

		// If no matchers match, we are done.
		break
	}

	// Default to mod time for now instead of parsing the date.
	data.date = data.modTime

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
func notesForExport(w *config.Website, byTag map[dd.Tag][]dd.NoteID) map[dd.NoteID]struct{} {
	e := map[dd.NoteID]struct{}{}

	// First, add all named notes from [[menu]] to the list.
	for _, m := range w.Menu {
		if mid, ok := m.(config.MenuNoteID); ok {
			e[mid.ID] = struct{}{}
		}
	}

	// Second, add all named notes from [[tags]] to the list.
	for _, t := range w.Tags {
		if len(t.ID) > 0 {
			e[t.ID] = struct{}{}
		}
	}

	// Third, add the feed's note ID if it's there.
	if len(w.Feed.ID) > 0 {
		e[w.Feed.ID] = struct{}{}
	}

	// Fourth, add the homepage note ID if it's there.
	if h, ok := w.Homepage.(config.HomepageNoteID); ok {
		e[h.ID] = struct{}{}
	}

	// Finally, add all notes with a publish tag from [[feed]] if it's there.
	if len(w.Feed.Tag) > 0 {
		for _, id := range byTag[w.Feed.Tag] {
			e[id] = struct{}{}
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
func modifyLinks(noteAst ast.Node, modify func(*ast.Link)) {
	ast.WalkFunc(noteAst, func(node ast.Node, entering bool) ast.WalkStatus {
		if link, ok := node.(*ast.Link); ok && entering {
			modify(link)
		}
		return ast.GoToNext
	})
}

// Load up the notes' content. Convention: note content is considered
// to start after the first blank line. So content is everything between
// the first blank line and EOF.
func (s *Store) readExportedContent(w *config.Website, notesDir string) error {
	p := map[dd.NoteID]note{}

	// Set up markdown parser.
	parserExtensions := parser.Tables | parser.FencedCode | parser.Strikethrough

	exportedNotes := notesForExport(w, s.byTag)

	// Load note content.
	for id := range exportedNotes {
		meta := s.meta[id]
		content, err := readContent(meta.filename, notesDir)
		if err != nil {
			return fmt.Errorf("failed to load note with ID '%s': %v", id, err)
		}

		// Parse note content with markdown parser.
		// https://github.com/gomarkdown/markdown/issues/280
		mp := parser.NewWithExtensions(parserExtensions)
		noteAst := mp.Parse(content)

		htmlFlags := html.CommonFlags | html.HrefTargetBlank
		opts := html.RendererOptions{Flags: htmlFlags}
		renderer := html.NewRenderer(opts)
		noteContent := markdown.Render(noteAst, renderer)

		/*
			// Modify the AST:
			//  - Replace note links with URLs.
			//  - Complain and quit if any linked notes are not published.
			//  - Collect any links out to files (distinguish .md links from files?).
			modifyLinks(noteAst, func(link *ast.Link) {
				linkStr := string(link.Destination)
				fmt.Println("Link:", linkStr)
				u, err := url.Parse(linkStr)
				if err != nil {
					// Not a URI, might be a note link.
					id, ok := w.IDFromLink(linkStr)
					if !ok {
						// Some weird link, continue.
						return
					}

					fmt.Println("OK, note ID:", id)
				}

				// Continue if the link is external.
				if u.IsAbs() {
					link.AdditionalAttributes = append(link.AdditionalAttributes, `target="_blank"`)
					return
				}

				// Here we only care if the link is a file.
			})
		*/

		p[id] = note{metadata: meta, doc: noteAst, content: noteContent}
	}

	s.pub = p
	return nil
}
