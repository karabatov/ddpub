package main

import (
	"flag"
	"fmt"
)

func main() {
	var notesDir = flag.String("notesDir", ".", "Directory that stores notes")
	flag.Parse()
	fmt.Println(*notesDir)
	flag.Usage()
}
