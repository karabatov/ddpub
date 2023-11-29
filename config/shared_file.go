package config

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
