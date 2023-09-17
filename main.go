package main

import (
	"bufio"
	"flag"
	"fmt"
	"net/url"
	"os"
	"path/filepath"
	"time"

	"github.com/gomarkdown/markdown/ast"
	"github.com/gomarkdown/markdown/parser"
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

	isNoteIDValidAndExists := func(id noteID) bool {
	}

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
