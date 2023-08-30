package main

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"regexp"

	"github.com/pelletier/go-toml/v2"
)

// DDConfig represents a configuration file for a single website.
type DDConfig struct {
	Address string
	Notes   struct {
		IdFormat string `toml:"id_format"`
	}
}

type noteID = string

type metadata struct {
	filename string
}

func main() {
	argsLen := len(os.Args[1:])
	// At least the command must be present.
	if argsLen == 0 {
		fmt.Println("Command is missing. Example:")
		fmt.Println("    ddpub check --config <dir> --notes <dir>")
		os.Exit(1)
	}
	command := os.Args[1]
	if command != "check" {
		fmt.Println("Only check command is supported")
		fmt.Println("    ddpub check --config <dir> --notes <dir>")
		os.Exit(1)
	}

	// Maybe refactor to `FlagSet` later, per command.

	var configDir = flag.String("config", ".", "Directory that has `config.toml`")
	var notesDir = flag.String("notes", ".", "Directory that stores notes")
	flag.Parse()
	fmt.Println(*configDir, *notesDir)

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

	fmt.Println("ID format:", cfg.Notes.IdFormat)

	// Read a list of “.md” files from the notes directory with names that match the regex.
	allFiles, err := os.ReadDir(*notesDir)
	if err != nil {
		fmt.Printf("Could not read the notes directory: %v", err)
		os.Exit(1)
	}

	notes := map[noteID]metadata{}

	var isMarkdownFile = regexp.MustCompile(".md$")
	for _, file := range allFiles {
		if file.IsDir() {
			continue
		}
		var name = file.Name()
		var id = validID.FindString(name)
		if isMarkdownFile.MatchString(name) && len(id) > 0 {
			var path = filepath.Join(*notesDir, name)
			data, err := readMetadata(path)
			if err != nil {
				fmt.Println("Could not read metadata from file:", name)
				continue
			}
			notes[id] = data
		}
	}

	// Print “Found N files.”
	fmt.Printf("Loaded metadata for %d notes.", len(notes))

	// Create a map of file ID to file metadata.
	// Read metadata for the files in the list:
	// * File ID
	// * Filename with extension (to be able to read it)
	// * File creation date
	// * Title (if present, defaults to blank)
	// * List of tags (if present, defaults to empty), with hashtag characters stripped
	// * Slug (if present, defaults to file ID)
	// * Date (if present, defaults to file modification date)
	// * Language (if present, defaults to default language code, currently "en-US")
	// Metadata is read until the first line that _isn't_ metadata, so it all must be at the beginning of the file.

	// Create a full list of unique tags (case-sensitive) present in the posts.
	// Create a map of tag to list of file IDs with that tag.

	// Verify the menu entries (loaded as part of config loading). The first `id`/`builtin`/`tag` entry (but not `url`) will be the homepage. (`[homepage]` from the sample config is obsolete)
	// Complain and exit if any `id` entries are not in the list of loaded files.
}

func readMetadata(path string) (metadata, error) {
	var data metadata
	return data, nil
}
