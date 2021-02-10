package main // import "github.com/caolo-game/caolo-backend"

import (
	"fmt"
	"github.com/caolo-game/caolo-backend/world_state"
	"github.com/gorilla/mux"
	"log"
	"net/http"
	"os"

	"github.com/thedevsaddam/renderer"

	"github.com/jmoiron/sqlx"
	_ "github.com/lib/pq"
)

type App struct {
	DB  *sqlx.DB
	rnd *renderer.Render
}

type Config struct {
	Port  string
	Host  string
	DbURI string
}

func NewConfig() *Config {
	return &Config{
		Port:  getEnv("PORT", "8000"),
		Host:  getEnv("HOST", "127.0.0.1"),
		DbURI: getEnv("DATABASE_URL", "postgres://postgres:admin@localhost:5432/caolo?sslmode=disable"),
	}
}

func getEnv(key string, defaultVal string) string {
	if value, exists := os.LookupEnv(key); exists {
		return value
	}
	return defaultVal
}

func NewApp(config *Config) *App {
	log.Println("Connecting to database")

	DB := sqlx.MustConnect("postgres", config.DbURI)
	rnd := renderer.New()
	return &App{DB, rnd}
}

func (a *App) GetGameConfig(w http.ResponseWriter, r *http.Request) {
	log.Println("game-config")

	state, err := world_state.GetLatestWorldState(a.DB)

	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	a.rnd.JSON(w, http.StatusOK, state.GameConfig)

}

func (a *App) GetRoomObjects(w http.ResponseWriter, req *http.Request) {
	log.Println("room-objects")

	q := req.URL.Query().Get("q")
	r := req.URL.Query().Get("r")

	if len(q) == 0 || len(r) == 0 {
		http.Error(w, "Expected q and r params. (Room id)", http.StatusBadRequest)
		return
	}

	state, err := world_state.GetLatestWorldState(a.DB)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	id := fmt.Sprintf("%s;%s", q, r)

	bots := state.Bots[id]
	structures := state.Structures[id]
	resources := state.Resources[id]

	pl := struct {
		Bots       interface{} `json:"bots"`
		Structures interface{} `json:"structures"`
		Resources  interface{} `json:"resources"`
	}{bots, structures, resources}

	resp := struct {
		Payload interface{} `json:"payload"`
		Time    interface{} `json:"time"`
	}{pl, state.Time}

	a.rnd.JSON(w, http.StatusOK, resp)
}

func (a *App) InitRouter() *mux.Router {
	r := mux.NewRouter()
	r.HandleFunc("/game-config", a.GetGameConfig).Methods("GET")
	r.HandleFunc("/room-objects", a.GetRoomObjects).Methods("GET")
	return r
}

func handleRequests() {
	config := NewConfig()
	app := NewApp(config)
	router := app.InitRouter()
	log.Println("Serving requests")
	err := http.ListenAndServe(fmt.Sprintf("%s:%s", config.Host, config.Port), router)
	log.Fatal(err)
}

func main() {
	log.Println("Caolo web")
	handleRequests()
}
