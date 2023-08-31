package main

import (
	"bufio"
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"time"

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
	fmt.Println(os.Args)
	argsLen := len(os.Args[1:])
	// At least the command must be present.
	if argsLen == 0 {
		fmt.Println("Command is missing. Example:")
		fmt.Println("    ddpub --check --config <dir> --notes <dir>")
		os.Exit(1)
	}
	command := os.Args[1]
	if command != "--check" {
		fmt.Println("Only check command is supported")
		fmt.Println("    ddpub --check --config <dir> --notes <dir>")
		os.Exit(1)
	}

	// Maybe refactor to `FlagSet` later, per command.

	checkCmd := flag.Bool("check", true, "Check the config")
	configDir := flag.String("config", ".", "Directory that has `config.toml`")
	notesDir := flag.String("notes", ".", "Directory that stores notes")
	flag.Parse()
	fmt.Println(*checkCmd, *configDir, *notesDir)

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

	// Create a map of file ID to file metadata.
	notes := map[noteID]metadata{}

	for _, file := range allFiles {
		if file.IsDir() {
			continue
		}

		var filename = file.Name()
		var id = validID.FindString(filename)
		if !matchMarkdownFile.MatchString(filename) || len(id) == 0 {
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

	// Verify the menu entries (loaded as part of config loading). The first `id`/`builtin`/`tag` entry (but not `url`) will be the homepage. (`[homepage]` from the sample config is obsolete)
	// Complain and exit if any `id` entries are not in the list of loaded files.
}

// Read metadata for the files in the list:
// * Filename with extension (to be able to read it)
// * File modification date
// * Title (if present, defaults to blank)
// * List of tags (if present, defaults to empty), with hashtag characters stripped
// * Slug (if present, defaults to file ID)
// * Date (if present, defaults to file modification date)
// * Language (if present, defaults to default language code, currently "en-US")
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
