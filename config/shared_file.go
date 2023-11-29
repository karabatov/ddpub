package config

import (
	"net/http"
	"os"
	"path/filepath"
)

// SharedFile is a file common to the whole website, embedded by default
// but can be overloaded if a certain file is present in the config dir.
type SharedFile struct {
	// Filename is the plain filename without any prefix.
	Filename string
	// Content is the actual contents of the file.
	Content []byte
	// ContentType goes into the HTTP header.
	ContentType string
}

// overload tries to read the file from configDir and replaces Content and ContentType.
func (sf *SharedFile) overload(configDir string) {
	path := filepath.Join(configDir, sf.Filename)
	f, err := os.ReadFile(path)
	if err != nil {
		// Don't overload if we get an error.
		return
	}

	sf.Content = f
	sf.ContentType = http.DetectContentType(f)
}
