package main

import (
	"flag"
	"fmt"
	"os"
	"time"

	"github.com/karabatov/ddpub/config"
	"github.com/karabatov/ddpub/notes"
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

	cfg, err := config.Load(*configDir)
	if err != nil {
		fmt.Printf("Couldn't load website config: %v\n", err)
		os.Exit(1)
	}

	store, err := notes.Load(cfg, *notesDir)
	if err != nil {
		fmt.Printf("Couldn't load notes: %v\n", err)
		os.Exit(1)
	}
	fmt.Printf("%v\n", store)

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
