package main // import "github.com/caolo-game/caolo-backend"

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"strconv"
	"strings"

	"github.com/caolo-game/caolo-backend/game_state"
	"github.com/gorilla/mux"

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

func (a *App) getGameConfig(w http.ResponseWriter, r *http.Request) {
	logHandlerEnter("game-config", r)

	state, err := game_state.GetLatestGameState(a.DB)

	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	a.rnd.JSON(w, http.StatusOK, state.GameConfig)

}

func (a *App) getRoomObjects(w http.ResponseWriter, req *http.Request) {
	logHandlerEnter("room-objects", req)

	q := req.URL.Query().Get("q")
	r := req.URL.Query().Get("r")

	if len(q) == 0 || len(r) == 0 {
		http.Error(w, "Expected q and r params. (Room id)", http.StatusBadRequest)
		return
	}

	state, err := game_state.GetLatestGameState(a.DB)
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

func (a *App) getRooms(w http.ResponseWriter, req *http.Request) {
	logHandlerEnter("rooms", req)
	state, err := game_state.GetLatestGameState(a.DB)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	rooms := state.Rooms
	var pl []interface{}
	for roomid, roompl := range rooms {
		roomqr := strings.Split(roomid, ";")
		q := roomqr[0]
		r := roomqr[1]
		qi, err := strconv.Atoi(q)
		if err != nil {
			panic(err)
		}
		ri, err := strconv.Atoi(r)
		if err != nil {
			panic(err)
		}
		rpl := roompl.(map[string]interface{})
		rpl["pos"] = struct {
			Q int `json:"q"`
			R int `json:"r"`
		}{qi, ri}

		pl = append(pl, rpl)
	}

	a.rnd.JSON(w, http.StatusOK, pl)
}

func (a *App) getHealth(w http.ResponseWriter, r *http.Request) {
	logHandlerEnter("health", r)
	a.rnd.NoContent(w)
}

func logHandlerEnter(name string, r *http.Request) {
	ip := r.RemoteAddr
	m := r.Method

	log.Printf("%s: [%s] %s", ip, m, name)
}

func (a *App) InitRouter() *mux.Router {
	r := mux.NewRouter()
	r.HandleFunc("/game-config", a.getGameConfig).Methods("GET")
	r.HandleFunc("/room-objects", a.getRoomObjects).Methods("GET")
	r.HandleFunc("/rooms", a.getRooms).Methods("GET")
	r.HandleFunc("/health", a.getHealth).Methods("GET")
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
