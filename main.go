package main

import (
	"bufio"
	"flag"
	"fmt"
	"net/url"
	"os"
	"path/filepath"
	"regexp"
	"time"

	"github.com/gomarkdown/markdown/ast"
	"github.com/gomarkdown/markdown/parser"
	"github.com/pelletier/go-toml/v2"
)

type noteID = string

type tag = string

type language = string

type metadata struct {
	filename string
	modTime  time.Time
	date     time.Time
	title    string
	slug     string
	tags     []tag
	language language
}

type PublicTag struct {
	Tag   tag
	ID    noteID `toml:"id"`
	Slug  string
	Title string
}

// MenuEntry always has a title and can be either of:
//   - builtin
//   - named note with id
//   - tag
//   - url
type MenuEntry struct {
	Title   string
	Builtin string
	ID      noteID `toml:"id"`
	Tag     tag
	URL     string `toml:"url"`
}

const (
	MenuInvalid = iota
	MenuBuiltinFeed
	MenuBuiltinSearch
	MenuBuiltinTags
	MenuNoteID
	MenuTag
	MenuURL
)

// Returns one of the Menu* const values.
func (m MenuEntry) kind() int {
	kind := MenuInvalid

	if len(m.Builtin) > 0 {
		switch m.Builtin {
		case "feed":
			kind = MenuBuiltinFeed
		case "search":
			kind = MenuBuiltinSearch
		case "tags":
			kind = MenuBuiltinTags
		default:
			return MenuInvalid
		}
	}

	// Can't verify if the note ID is valid or not here.
	if len(m.ID) > 0 && kind == MenuInvalid {
		kind = MenuNoteID
	} else {
		return MenuInvalid
	}

	if len(m.Tag) > 0 && kind == MenuInvalid {
		kind = MenuTag
	} else {
		return MenuInvalid
	}

	if len(m.URL) > 0 && kind == MenuInvalid {
		kind = MenuURL
	} else {
		return MenuInvalid
	}

	return kind
}

// DDConfig represents a configuration file for a single website.
type DDConfig struct {
	Address string
	Feed    struct {
		Tag tag
	}
	Menu  []MenuEntry
	Notes struct {
		IdFormat     string `toml:"id_format"`
		IdLinkFormat string `toml:"id_link_format"`
	}
	Tags []PublicTag
}

type note struct {
	meta metadata
	doc  ast.Node
}

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

func main() {
	startTime := time.Now()
	fmt.Println(os.Args)

	// Maybe refactor to `FlagSet` later, per command.

	checkCmd := flag.Bool("check", false, "Check the config")
	serveCmd := flag.Bool("serve", false, "Serve the notes")
	configDir := flag.String("config", ".", "Directory that has `config.toml`")
	notesDir := flag.String("notes", ".", "Directory that stores notes")
	flag.Parse()

	if !(*checkCmd || *serveCmd) {
		fmt.Println("Command is missing. Example:")
		fmt.Println("    ddpub --check --config <dir> --notes <dir>")
		os.Exit(1)
	}

	// Try to read the config file.
	*configDir = filepath.Clean(*configDir)
	configPath := filepath.Join(*configDir, "config.toml")
	configFile, err := os.ReadFile(configPath)
	if err != nil {
		fmt.Printf("Could not open config file '%s':\n\t%v", configPath, err)
		os.Exit(1)
	}

	var cfg DDConfig
	err = toml.Unmarshal(configFile, &cfg)
	if err != nil {
		fmt.Printf("Could not read config file '%s':\n\t%v", configPath, err)
		os.Exit(1)
	}
	fmt.Println("address:", cfg.Address)

	// Load file ID regex from config and try to compile.
	validID, err := regexp.Compile(cfg.Notes.IdFormat)
	if err != nil {
		fmt.Printf("Could not compile id_format regular expression '%s': %v", cfg.Notes.IdFormat, err)
		os.Exit(1)
	}

	isValidNoteID := func(test noteID) bool {
		var id = validID.FindString(test)
		return len(id) > 0 && id == test
	}

	fmt.Println("ID format:", cfg.Notes.IdFormat)

	// Load ID link format regex from config and try to compile.
	idLinkFormat, err := regexp.Compile(cfg.Notes.IdLinkFormat)
	if err != nil {
		fmt.Printf("Could not compile id_link_format regular expression '%s': %v", cfg.Notes.IdFormat, err)
		os.Exit(1)
	}
	fmt.Println("ID link format:", cfg.Notes.IdLinkFormat)

	idFromLink := func(link string) (noteID, bool) {
		id, ok := firstSubmatch(idLinkFormat, link)
		if !ok {
			return "", false
		}
		return id, isValidNoteID(id)
	}

	// Read a list of “.md” files from the notes directory with names that match the regex.
	allFiles, err := os.ReadDir(*notesDir)
	if err != nil {
		fmt.Printf("Could not read the notes directory: %v", err)
		os.Exit(1)
	}

	// Create a map of file ID to file metadata.
	notes := map[noteID]metadata{}

	for _, file := range allFiles {
		if file.IsDir() {
			continue
		}

		var filename = file.Name()
		var id = validID.FindString(filename)
		if !matchMarkdownFile.MatchString(filename) || !isValidNoteID(id) {
			continue
		}

		fileMetadata, err := readMetadata(filename, *notesDir)
		if err != nil {
			fmt.Println("Could not read metadata from file:", filename)
			continue
		}

		notes[id] = fileMetadata
	}

	fmt.Printf("Loaded metadata for %d notes.\n", len(notes))

	// Create a full list of unique tags (case-sensitive) present in the posts.
	// Create a map of tag to list of file IDs with that tag.
	notesByTag := map[tag][]noteID{}
	for id, data := range notes {
		for _, t := range data.tags {
			if tags, ok := notesByTag[t]; ok {
				notesByTag[t] = append(tags, id)
			} else {
				notesByTag[t] = []tag{id}
			}
		}
	}
	fmt.Printf("Loaded %d internal tags.\n", len(notesByTag))

	isNoteIDValidAndExists := func(id noteID) bool {
		if !isValidNoteID(id) {
			return false
		}
		_, ok := notes[id]
		return ok
	}

	// Verify the published tags before menu entries. Menu entries can link to tags,
	// but only published tags are allowed (all other tags are stripped).
	publishedTags := []PublicTag{}
	for _, t := range cfg.Tags {
		// At least the tag must be present.
		if len(t.Tag) == 0 {
			fmt.Println("Error in [[tags]]: cannot publish empty tag.")
			os.Exit(1)
		}
		// Default the slug to the tag itself.
		if len(t.Slug) == 0 {
			t.Slug = t.Tag
		}
		// Default the title to the mapped tag.
		if len(t.Title) == 0 {
			t.Title = t.Slug
		}
		// If note ID is present, it must be valid.
		if len(t.ID) > 0 && !isNoteIDValidAndExists(t.ID) {
			fmt.Printf("Invalid or non-existing note ID '%s' in published tag '%s'.", t.ID, t.Tag)
			os.Exit(1)
		}
		// Tag description is good.
		publishedTags = append(publishedTags, t)
	}

	fmt.Printf("Loaded %d published tags.\n", len(publishedTags))

	// Verify the menu entries (loaded as part of config loading). The first `id`/`builtin`/`tag` entry (but not `url`) will be the homepage. (`[homepage]` from the sample config is obsolete)
	// Complain and exit if any `id` entries or tags are not in the list of loaded files.
	menu := []MenuEntry{}

	for _, m := range cfg.Menu {
		if len(m.Title) == 0 {
			fmt.Println("Error in [[menu]]: entry title cannot be empty.")
			os.Exit(1)
		}
		switch m.kind() {
		case MenuInvalid:
			fmt.Printf("Invalid menu entry '%s'.", m.Title)
			os.Exit(1)
		case MenuBuiltinFeed, MenuBuiltinSearch, MenuBuiltinTags:
			// OK
		case MenuNoteID:
			if !isNoteIDValidAndExists(m.ID) {
				fmt.Printf("Invalid or non-existing note ID '%s' in menu entry '%s'.", m.ID, m.Title)
				os.Exit(1)
			}
		case MenuTag:
			exists := false
			for i := range publishedTags {
				if publishedTags[i].Slug == m.Tag {
					exists = true
					break
				}
			}
			if !exists {
				fmt.Printf("Error: tag '%s' in menu entry '%s' must be published in [[tags]].", m.Tag, m.Title)
				os.Exit(1)
			}
		case MenuURL:
			// OK
		}
		menu = append(menu, m)
	}

	fmt.Printf("Loaded %d menu entries.\n", len(menu))

	// Build the complete list of *known* note IDs to be published before parsing).
	// They are all valid, verified and exist in `notes`.
	exportedNotes := map[noteID]bool{}

	// First, add all named notes from [[menu]] to the list.
	for _, m := range menu {
		if m.kind() == MenuNoteID {
			exportedNotes[m.ID] = true
		}
	}
	// Second, add all named notes from [[tags]] to the list.
	for _, t := range publishedTags {
		if len(t.ID) > 0 {
			exportedNotes[t.ID] = true
		}
	}
	// Finally, add all notes with a publish tag from [[feed]] if it's there.
	if len(cfg.Feed.Tag) > 0 {
		for _, id := range notesByTag[cfg.Feed.Tag] {
			exportedNotes[id] = true
		}
	}

	fmt.Printf("Preparing to publish %d notes…\n", len(exportedNotes))

	// Load up the notes' content. Convention: note content is considered
	// to start after the first blank line. So content is everything between
	// the first blank line and EOF.
	parsedNotes := map[noteID]note{}

	// Set up markdown parser.
	parserExtensions := parser.Tables | parser.FencedCode | parser.Strikethrough

	// Load note content.
	for id := range exportedNotes {
		meta := notes[id]
		content, err := readContent(meta.filename, *notesDir)
		if err != nil {
			fmt.Printf("Failed to load note with ID '%s': %v", id, err)
			os.Exit(1)
		}

		// Parse note content with markdown parser.
		// https://github.com/gomarkdown/markdown/issues/280
		mp := parser.NewWithExtensions(parserExtensions)
		noteAst := mp.Parse(content)

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
				id, ok := idFromLink(linkStr)
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

		parsedNotes[id] = note{meta: meta, doc: noteAst}
	}

	fmt.Printf("Loaded %d notes.\n", len(parsedNotes))

	// At this point the surface check is complete! There may be more
	// errors like duplicate tags or bad URLs, but these will be caught later.
	fmt.Printf("Config OK. Startup took %v.", time.Since(startTime))
	if *checkCmd && !*serveCmd {
		os.Exit(0)
	}
	if !*serveCmd {
		os.Exit(1)
	}
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
func readMetadata(filename, directory string) (metadata, error) {
	var path = filepath.Join(directory, filename)
	var data metadata

	file, err := os.Open(path)
	if err != nil {
		return data, err
	}
	defer file.Close()

	data.filename = filename
	data.modTime = fileModTime(file)

	s := bufio.NewScanner(file)
	for s.Scan() {
		if title, ok := firstSubmatch(matchLineTitle, s.Text()); ok {
			data.title = title
			continue
		}

		if tags, ok := firstSubmatch(matchLineTags, s.Text()); ok {
			data.tags = tagsFromLine(tags)
			continue
		}

		if slug, ok := firstSubmatch(matchLineSlug, s.Text()); ok {
			data.slug = slug
			continue
		}

		if lang, ok := firstSubmatch(matchLineLanguage, s.Text()); ok {
			// TODO: Add validation.
			data.language = lang
			continue
		}

		if _, ok := firstSubmatch(matchLineDate, s.Text()); ok {
			// TODO: Define date format and parse.
			continue
		}

		// If no matchers match, we are done.
		break
	}

	// Default to mod time for now instead of parsing the date.
	data.date = data.modTime

	return data, nil
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

func fileModTime(file *os.File) time.Time {
	if stat, err := file.Stat(); err == nil {
		return stat.ModTime()
	} else {
		return time.Now()
	}
}

func firstSubmatch(re *regexp.Regexp, line string) (string, bool) {
	if matches := re.FindStringSubmatch(line); len(matches) > 1 {
		return matches[1], true
	}

	return "", false
}

func tagsFromLine(line string) []tag {
	tags := []tag{}
	for _, tagPair := range matchOneTag.FindAllStringSubmatch(line, -1) {
		tags = append(tags, tagPair[1])
	}
	return tags
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
