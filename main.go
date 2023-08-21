package main

import (
	"flag"
	"fmt"

	"github.com/pelletier/go-toml/v2"
)

type DDConfig struct {
	Address string
}

func main() {
	var notesDir = flag.String("notesDir", ".", "Directory that stores notes")
	flag.Parse()
	fmt.Println(*notesDir)
	flag.Usage()

	static := `
	address = "ddpub.org"
	`

	var cfg DDConfig
	err := toml.Unmarshal([]byte(static), &cfg)
	if err != nil {
		panic(err)
	}
	fmt.Println("address:", cfg.Address)
}
