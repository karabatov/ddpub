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
}
