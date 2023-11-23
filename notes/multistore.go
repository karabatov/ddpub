package notes

import (
	"github.com/karabatov/ddpub/config"
	"github.com/karabatov/ddpub/dd"
)

type MultiStore struct {
	Main      *Store
	SubStores map[dd.Language]*Store
}

func NewMultiStore(w *config.Website, notesDir string) (*MultiStore, error) {
	var m MultiStore

	mainStore, err := newStore(w.Main, notesDir)
	if err != nil {
		return nil, err
	}
	m.Main = mainStore

	m.SubStores = make(map[dd.Language]*Store)
	for _, cfg := range w.SubConfigs {
		s, err := newStore(cfg, notesDir)
		if err != nil {
			return nil, err
		}
		m.SubStores[cfg.Language.Code] = s
	}

	return &m, nil
}
