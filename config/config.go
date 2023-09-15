// Package config loads and validates website config from a directory.
package config

import "fmt"

// Website represents the configuration of a website.
type Website struct {
}

func Load(configDir string) (Website, error) {
	return Website{}, fmt.Errorf("not implemented")
}
