package main

import (
	// "encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"

	"github.com/gorilla/websocket"

	"github.com/jmoiron/sqlx"
	_ "github.com/lib/pq"
)

type App struct {
	DB     *sqlx.DB
	Config *Config
}

type Config struct {
	Port  string
	Host  string
	DbUri string
}

func getEnv(key string, defaultVal string) string {
	if value, exists := os.LookupEnv(key); exists {
		return value
	}
	return defaultVal
}

func NewConfig() *Config {
	return &Config{
		Port:  getEnv("PORT", "8000"),
		Host:  getEnv("HOST", "127.0.0.1"),
		DbUri: getEnv("DATABASE_URL", "postgres://postgres:admin@localhost:5432/caolo?sslmode=disable"),
	}
}

func NewApp(config *Config) *App {
	log.Print("Connecting to Database")

	DB := sqlx.MustConnect("postgres", config.DbUri)

	return &App{DB, config}
}

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool { return true },
}

func (*App) gameState(w http.ResponseWriter, r *http.Request) {
	c, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Print("upgrade:", err)
		return
	}
	defer c.Close()
	for {
		mt, message, err := c.ReadMessage()
		if err != nil {
			log.Println("read:", err)
			break
		}
		log.Printf("recv: %s", message)
		err = c.WriteMessage(mt, message)
		if err != nil {
			log.Println("write:", err)
			break
		}
	}
}

func main() {
	config := NewConfig()
	app := NewApp(config)

	http.HandleFunc("/world/object-stream", app.gameState)

	log.Print("Service is listening for connections...")
	log.Fatal(http.ListenAndServe(fmt.Sprintf("%s:%s", config.Host, config.Port), nil))
}
