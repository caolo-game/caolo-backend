package client

import (
	"github.com/caolo-game/cao-rt/world"
	"github.com/gorilla/websocket"
)

type Client struct {
	Conn   *websocket.Conn
	RoomId world.RoomId
}

func NewClient(conn *websocket.Conn) Client {
	return Client{
		Conn: conn,
		RoomId: world.RoomId{
			Q: -1,
			R: -1,
		},
	}
}
