package ws

import (
	"bytes"
	"encoding/json"
	"log"
	"net/http"
	"time"

	"github.com/caolo-game/cao-rt/world"
	"github.com/gorilla/websocket"
)

// Single Client handler
type Client struct {
	Conn   *websocket.Conn
	Hub    *GameStateHub
	RoomId world.RoomId
	Send   chan []byte
}

func NewClient(conn *websocket.Conn, hub *GameStateHub) Client {
	return Client{
		Conn:   conn,
		Hub:    hub,
		RoomId: world.RoomId{Q: -1, R: -1},
		Send:   make(chan []byte),
	}
}

type InputMsg struct {
	Ty     string       `json:"ty"`
	RoomId world.RoomId `json:"room_id,omitempty"`
}

func (c *Client) readPump() {
	defer func() {
		c.Hub.Unregister <- c
		c.Conn.Close()
	}()
	c.Conn.SetReadLimit(256)
	c.Conn.SetReadDeadline(time.Now().Add(60 * time.Second))
	c.Conn.SetPongHandler(func(string) error { c.Conn.SetReadDeadline(time.Now().Add(60 * time.Second)); return nil })
	for {
		_, msg, err := c.Conn.ReadMessage()
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				log.Printf("Client going away: %v", err)
			}
			return
		}
		msg = bytes.TrimSpace(bytes.Replace(msg, []byte{'\n'}, []byte{' '}, -1))
		var pl InputMsg
		err = json.Unmarshal(msg, &pl)
		if err != nil {
			log.Panicf("Invalid message %v", err)
			return
		}
		switch pl.Ty {
		case "room_id":
			c.RoomId = pl.RoomId
		default:
			log.Printf("Unhandled msg type %v", pl.Ty)
		}
	}
}

func (c *Client) writePump() {
	ticker := time.NewTicker(50 * time.Second)
	defer func() {
		ticker.Stop()
		c.Conn.Close()
	}()

	for {
		select {
		case message, ok := <-c.Send:
			c.Conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if !ok {
				// hub closed this channel
				c.Conn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}
			w, err := c.Conn.NextWriter(websocket.TextMessage)
			if err != nil {
				return
			}
			w.Write(message)
		case <-ticker.C:
			c.Conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if err := c.Conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		}
	}
}

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin: func(r *http.Request) bool {
		return true
	},
}

func ServeWs(hub *GameStateHub, w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("Failed to upgrade ws connection %v", err)
	}
	client := NewClient(conn, hub)
	hub.Register <- &client

	log.Println("New client")

	go client.writePump()
	go client.readPump()
}
