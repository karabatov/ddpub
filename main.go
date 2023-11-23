package main

import (
	"flag"
	"fmt"
	"log"
	"net/http"
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

	cfg, err := config.New(configDir)
	if err != nil {
		log.Fatalf("Couldn't load website config: %v\n", err)
	}

	store, err := notes.NewMultiStore(cfg, notesDir)
	if err != nil {
		log.Fatalf("Couldn't load notes: %v\n", err)
	}

	router, err := notes.NewMultiRouter(cfg, store)
	if err != nil {
		log.Fatalf("Could not create router: %s", err)
	}

	// At this point the surface check is complete! There may be more
	// errors like duplicate tags or bad URLs, but these will be caught later.
	log.Printf("Config OK. Startup took %v.", time.Since(startTime))

	// If we were only checking the config, exit now.
	if os.Args[1] == "check" {
		os.Exit(0)
	}

	// Serve notes.
	log.Println("Starting server...")
	log.Printf("In your browser, open: http://localhost:%d", port)

	srv := &http.Server{
		Addr:         fmt.Sprintf(":%d", port),
		Handler:      router.ServeMux(),
		ReadTimeout:  30 * time.Second,
		WriteTimeout: 30 * time.Second,
	}
	log.Fatal(srv.ListenAndServe())
}
