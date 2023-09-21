package notes

import (
	"fmt"
	"log"
	"net/http"

	"github.com/karabatov/ddpub/config"
)

func NewServeMux(w *config.Website, s *Store) (*http.ServeMux, error) {
	mux := http.NewServeMux()

	addHandler(mux, "/", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintf(w, "Hello from DDPub")
	})

	addHandler(mux, "/", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintf(w, "Hello from DDPub")
	})

	return mux, nil
}

func addHandler(mux *http.ServeMux, pattern string, handler http.HandlerFunc) {
	defer func() {
		if err := recover(); err != nil {
			log.Fatalf("Could not create router: %v", err)
		}
	}()

	mux.HandleFunc(pattern, handler)
}
