package notes

import (
	"log"
	"net/http"

	"github.com/karabatov/ddpub/config"
)

type Router struct {
	routes map[string]http.HandlerFunc
}

func NewRouter(w *config.Website, s *Store) (*Router, error) {
	r := Router{routes: make(map[string]http.HandlerFunc)}

	return &r, nil
}

func (r Router) ServeMux() *http.ServeMux {
	mux := http.NewServeMux()

	for k, v := range r.routes {
		addHandler(mux, k, v)
	}

	return mux
}

func addHandler(mux *http.ServeMux, pattern string, handler http.HandlerFunc) {
	defer func() {
		// We shouldn't panic here, but it's better than crashing.
		if err := recover(); err != nil {
			log.Fatalf("Could not create router: %v", err)
		}
	}()

	mux.HandleFunc(pattern, handler)
}
