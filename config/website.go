package config

//go:embed theme.css
var themeCSS []byte

// Website represents the configuration of a website.
type Website struct {
	Main       *WebsiteLang
	SubConfigs []*WebsiteLang
	ThemeCSS   []byte
}

func New(configDir string) (*Website, error) {
	var w Website

	w.ThemeCSS = themeCSS

	return &w, nil
}
