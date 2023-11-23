package config

// Website represents the configuration of a website.
type Website struct {
	Main       *WebsiteLang
	SubConfigs []*WebsiteLang
}
