package main

import (
	"bytes"
	"encoding/json"
	"log"
	"net/http"
	"time"

	"github.com/gorilla/websocket"
)

// Single client handler
type client struct {
	conn        *websocket.Conn
	hub         *GameStateHub
	roomIds     []RoomId
	entities    chan *RoomState
	onNewRoomId chan RoomId
}

func NewClient(conn *websocket.Conn, hub *GameStateHub) client {
	return client{
		conn:        conn,
		hub:         hub,
		roomIds:     []RoomId{},
		entities:    make(chan *RoomState),
		onNewRoomId: make(chan RoomId),
	}
}

type InputMsg struct {
	Ty     string `json:"ty"`
	RoomId RoomId `json:"room_id,omitempty"`
}

func FindRoomIdIndex(arr []RoomId, key RoomId) int {
	for i := range arr {
		if arr[i] == key {
			return i
		}
	}
	return -1
}

/// Might move the last item so delete doesn't trigger a reordering of elements
func RemoveRoomId(arr []RoomId, key RoomId) []RoomId {
	index := FindRoomIdIndex(arr, key)
	if index < 0 {
		return arr
	}

	// swap with the last
	arr[index] = arr[len(arr)-1]

	return arr[:len(arr)-1]
}

func (c *client) readPump() {
	defer func() {
		c.hub.unregister <- c
		c.conn.Close()
	}()
	c.conn.SetReadLimit(256)
	c.conn.SetReadDeadline(time.Now().Add(60 * time.Second))
	c.conn.SetPongHandler(func(string) error { c.conn.SetReadDeadline(time.Now().Add(60 * time.Second)); return nil })
	for {
		_, msg, err := c.conn.ReadMessage()
		if err != nil {
			log.Printf("Client going away: %v", err)
			return
		}
		msg = bytes.TrimSpace(bytes.Replace(msg, []byte{'\n'}, []byte{' '}, -1))
		var pl InputMsg
		err = json.Unmarshal(msg, &pl)
		if err != nil {
			log.Printf("Invalid message %v", err)
			return
		}
		switch pl.Ty {
		case "room_id":
			if len(c.roomIds) > 100 {
				log.Print("Client is listening to too many roomIds")
				continue
			}
			c.roomIds = append(c.roomIds, pl.RoomId)
			c.onNewRoomId <- pl.RoomId
		case "unsubscribe_room_id":
			c.roomIds = RemoveRoomId(c.roomIds, pl.RoomId)
		default:
			log.Printf("Unhandled msg type %v", pl.Ty)
		}
	}
}

type Response struct {
	Ty      string      `json:"ty"`
	Payload interface{} `json:"payload"`
}

func sendJson(conn *websocket.Conn, ty string, payload interface{}) error {
	response := Response{
		Ty:      ty,
		Payload: payload,
	}
	pl, err := json.Marshal(response)
	if err != nil {
		log.Fatalf("Failed to serialize terrain payload: %v", err)
	}
	w, err := conn.NextWriter(websocket.TextMessage)
	if err != nil {
		return err
	}
	w.Write(pl)

	return nil
}

func (c *client) writePump() {
	ticker := time.NewTicker(50 * time.Second)
	defer func() {
		ticker.Stop()
		c.conn.Close()
	}()

	for {
		select {
		case roomId, ok := <-c.onNewRoomId:
			if !ok {
				c.conn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}
			terrain := c.hub.Terrain[roomId]
			err := sendJson(c.conn, "terrain", terrain)
			if err != nil {
				return
			}
			entities := c.hub.Entities[roomId]
			err = sendJson(c.conn, "entities", entities)
			if err != nil {
				return
			}
		case entities, ok := <-c.entities:
			c.conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if !ok {
				// hub closed this channel
				c.conn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}
			err := sendJson(c.conn, "entities", entities)
			if err != nil {
				return
			}
		case <-ticker.C:
			c.conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if err := c.conn.WriteMessage(websocket.PingMessage, nil); err != nil {
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
	hub.register <- &client

	log.Println("New client")

	go client.writePump()
	go client.readPump()
}
