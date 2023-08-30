package main

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"

	"github.com/pelletier/go-toml/v2"
)

// DDConfig represents a configuration file for a single website.
type DDConfig struct {
	Address string
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
	// Read a list of “.md” files from the notes directory with names that match the regex.
	// Print “Found N files.”

	// Create a map of file ID to file metadata.
	// Read metadata for the files in the list:
	// * File ID
	// * Filename with extension (to be able to read it)
	// * File creation date
	// * Title (if present, defaults to blank)
	// * List of tags (if present, defaults to empty), with hashtag characters stripped
	// * Slug (if present, defaults to file ID)
	// * Date (if present, defaults to file creation date)
	// * Language (if present, defaults to default language code, currently "en-US")
	// Metadata is read until the first line that _isn't_ metadata, so it all must be at the beginning of the file.

	// Create a full list of unique tags (case-sensitive) present in the posts.
	// Create a map of tag to list of file IDs with that tag.

	// Verify the menu entries (loaded as part of config loading). The first `id`/`builtin`/`tag` entry (but not `url`) will be the homepage. (`[homepage]` from the sample config is obsolete)
	// Complain and exit if any `id` entries are not in the list of loaded files.
}
