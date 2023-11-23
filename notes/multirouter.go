package notes

import (
	"log"
	"net/http"

	"github.com/karabatov/ddpub/config"
)

type MultiRouter struct {
	main       *Router
	subRouters []*Router
}

func NewMultiRouter(w *config.Website, m *MultiStore) (*MultiRouter, error) {
	var mr MultiRouter

	mainRouter, err := newRouter(w.Main, m.Main)
	if err != nil {
		return nil, err
	}
	mr.main = mainRouter

	mr.subRouters = make([]*Router, 0)
	for _, cfg := range w.SubConfigs {
		router, err := newRouter(cfg, m.SubStores[cfg.Language.Code])
		if err != nil {
			return nil, err
		}
		mr.subRouters = append(mr.subRouters, router)
	}

	return &mr, nil
}

func (mr MultiRouter) ServeMux() *http.ServeMux {
	defer func() {
		// We shouldn't panic here, but it's better than crashing.
		if err := recover(); err != nil {
			log.Fatalf("Could not create router: %v", err)
		}
	}()

	mux := http.NewServeMux()
	register(mux, mr.main)

	for _, sub := range mr.subRouters {
		register(mux, sub)
	}

	return mux
}

func register(mux *http.ServeMux, r *Router) {
	for pattern, handler := range r.routes {
		mux.HandleFunc(pattern, handler)
	}
}
