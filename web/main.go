package main // import "github.com/caolo-game/caolo-backend/web"

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"strconv"
	"strings"

	"github.com/caolo-game/caolo-backend/game_state"

	"github.com/gorilla/handlers"
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
	DbUri string
}

func NewConfig() *Config {
	return &Config{
		Port:  getEnv("PORT", "8000"),
		Host:  getEnv("HOST", "127.0.0.1"),
		DbUri: getEnv("DATABASE_URL", "postgres://postgres:admin@localhost:5432/caolo?sslmode=disable"),
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

	DB := sqlx.MustConnect("postgres", config.DbUri)
	rnd := renderer.New()
	return &App{DB, rnd}
}

func (a *App) getGameConfig(w http.ResponseWriter, r *http.Request) {

	state, err := game_state.GetLatestGameState(a.DB)

	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	a.rnd.JSON(w, http.StatusOK, state.GameConfig)

}

func (a *App) getRoomObjects(w http.ResponseWriter, req *http.Request) {

	query := req.URL.Query()

	q := query.Get("q")
	r := query.Get("r")

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
	a.rnd.NoContent(w)
}

func (a *App) getRoomTerrain(w http.ResponseWriter, req *http.Request) {
	query := req.URL.Query()

	q := query.Get("q")
	r := query.Get("r")

	const getRoomQuery = `
SELECT objects.value AS room
FROM public.world_output t, jsonb_each(payload -> 'terrain') objects
WHERE objects.key::text = $1
ORDER BY t.created DESC
`

	type QResult struct {
		Room []byte `db:"room"`
	}

	roomId := fmt.Sprintf("%s;%s", q, r)
	var result QResult
	err := a.DB.Get(&result, getRoomQuery, roomId)
	if err != nil {
		if err.Error() == "sql: no rows in result set" {
			http.Error(w, "Room not found", http.StatusNotFound)
			return
		}
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	w.WriteHeader(http.StatusOK)
	w.Write(result.Room)
	w.Header().Add("content-type", "applcation/json")
}

func (a *App) InitRouter() *mux.Router {
	r := mux.NewRouter()
	r.HandleFunc("/game-config", a.getGameConfig).Methods("GET")
	r.HandleFunc("/room-objects", a.getRoomObjects).Methods("GET")
	r.HandleFunc("/rooms", a.getRooms).Methods("GET")
	r.HandleFunc("/health", a.getHealth).Methods("GET")
	r.HandleFunc("/terrain", a.getRoomTerrain).Methods("GET")
	return r
}

func handleRequests() {
	config := NewConfig()
	app := NewApp(config)
	router := app.InitRouter()
	log.Println("Serving requests")
	recoveryRouter := handlers.RecoveryHandler()(router)
	compressedRouter := handlers.CompressHandler(recoveryRouter)
	loggedRouter := handlers.CombinedLoggingHandler(os.Stdout, compressedRouter)
	corsedRouter := handlers.CORS(handlers.AllowCredentials(), handlers.AllowedOrigins([]string{"*"}))(loggedRouter)
	err := http.ListenAndServe(fmt.Sprintf("%s:%s", config.Host, config.Port), corsedRouter)
	log.Fatal(err)
}

func main() {
	log.Println("Caolo web")
	handleRequests()
}
