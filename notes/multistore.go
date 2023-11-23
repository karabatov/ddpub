package notes

import "github.com/karabatov/ddpub/config"

type MultiStore struct {
	Main      *Store
	SubStores []*Store
}

func NewMultiStore(w *config.Website, notesDir string) (*MultiStore, error) {
	var m MultiStore

	mainStore, err := newStore(w.Main, notesDir)
	if err != nil {
		return nil, err
	}
	m.Main = mainStore

	m.SubStores = make([]*Store, len(w.SubConfigs))
	for _, cfg := range w.SubConfigs {
		s, err := newStore(cfg, notesDir)
		if err != nil {
			return nil, err
		}
		m.SubStores = append(m.SubStores, s)
	}

	return &m, nil
}
