package main // import "github.com/caolo-game/caolo-backend"

import (
	"encoding/json"
	"fmt"
	"github.com/caolo-game/caolo-backend/world_state"
	"github.com/gorilla/mux"
	"log"
	"net/http"
	"os"

	"github.com/jmoiron/sqlx"
	_ "github.com/lib/pq"
)

var WorldStateQuery = `
SELECT t.payload
FROM world_output t
ORDER BY t.created DESC
LIMIT 1
`

type App struct {
	DB *sqlx.DB
}

type Config struct {
	Port string
	Host string
    DbURI string
}

func NewConfig() *Config {
	return &Config{
		Port: getEnv("PORT", "8000"),
		Host: getEnv("HOST", "127.0.0.1"),
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
	log.Print("Connecting to database")

	DB := sqlx.MustConnect("postgres", config.DbURI)
	return &App{DB}
}

func (a *App) GetGameConfig(w http.ResponseWriter, r *http.Request) {
	log.Print("gameConfig")

	type WorldQResult struct {
		Payload []byte `db:"payload"`
	}

	results := []WorldQResult{}
	a.DB.Select(&results, WorldStateQuery)
    if len(results) == 0 {
		http.Error(w, "No world state", http.StatusInternalServerError)
		return
    }

	var state world_state.WorldState
	err := json.Unmarshal(results[0].Payload, &state)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	resp, err := json.Marshal(state.GameConfig)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.Write(resp)

}

func (a *App) InitRouter() *mux.Router {
	r := mux.NewRouter()
	r.HandleFunc("/game-config", a.GetGameConfig).Methods("GET")
	return r
}

func handleRequests() {
	config := NewConfig()
	app := NewApp(config)
	router := app.InitRouter()
	log.Fatal(http.ListenAndServe(fmt.Sprintf("%s:%s", config.Host, config.Port), router))
}

func main() {
	fmt.Println("Caolo web")
	handleRequests()
}
