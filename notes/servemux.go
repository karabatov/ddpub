package notes

import (
	"fmt"
	"net/http"

	"github.com/karabatov/ddpub/config"
)

func NewServeMux(w *config.Website, s *Store) *http.ServeMux {
	mux := http.NewServeMux()

	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintf(w, "Hello from DDPub")
	})

	return mux
}
