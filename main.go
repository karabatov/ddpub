package main

import (
	"flag"
	"fmt"
	"os"
	"time"

	"github.com/karabatov/ddpub/config"
	"github.com/karabatov/ddpub/notes"
)

const (
	usage = `ddpub is a tool to serve one set of notes as many websites.

	Check config:
			ddpub check --config <dir> --notes <dir>

	Serve notes:
			ddpub serve --config <dir> --notes <dir> --port <port>`
)

var (
	configDir string
	notesDir  string
	port      int
)

func main() {
	startTime := time.Now()

	if len(os.Args) < 3 {
		fmt.Println(usage)
		os.Exit(1)
	}

	const (
		configFlag  = "config"
		configUsage = "Directory that has `config.toml`"
		notesFlag   = "notes"
		notesUsage  = "Directory that stores notes"
	)

	switch os.Args[1] {
	case "check":
		check := flag.NewFlagSet("check", flag.ExitOnError)
		check.Usage = func() { fmt.Println(usage) }
		check.StringVar(&configDir, configFlag, ".", configUsage)
		check.StringVar(&notesDir, notesFlag, ".", notesUsage)
		check.Parse(os.Args[2:])
	case "serve":
		serve := flag.NewFlagSet("serve", flag.ExitOnError)
		serve.Usage = func() { fmt.Println(usage) }
		serve.StringVar(&configDir, configFlag, ".", configUsage)
		serve.StringVar(&notesDir, notesFlag, ".", notesUsage)
		serve.IntVar(&port, "port", 44234, "Port to serve notes")
		serve.Parse(os.Args[2:])
	default:
		fmt.Println(usage)
		os.Exit(1)
	}

	cfg, err := config.Load(configDir)
	if err != nil {
		fmt.Printf("Couldn't load website config: %v\n", err)
		os.Exit(1)
	}

	store, err := notes.Load(cfg, notesDir)
	if err != nil {
		fmt.Printf("Couldn't load notes: %v\n", err)
		os.Exit(1)
	}
	fmt.Printf("%v\n", store)

	// At this point the surface check is complete! There may be more
	// errors like duplicate tags or bad URLs, but these will be caught later.
	fmt.Printf("Config OK. Startup took %v.", time.Since(startTime))

	// If we were only checking the config, exit now.
	if os.Args[1] == "check" {
		os.Exit(0)
	}

	// Serve notes.
}
