package ws

import (
	"encoding/json"
	"log"

	cao_world "github.com/caolo-game/cao-rt/cao_world_pb"

	"github.com/caolo-game/cao-rt/world"
)

type GameStateHub struct {
	Rooms   map[world.RoomId]RoomState
	Clients map[*Client]bool

	// push new worldState to hub
	WorldState chan *cao_world.RoomEntities

	/// register new Clients
	Register chan *Client
	/// un-register new Clients
	Unregister chan *Client
}

type RoomState struct {
	Time       int64                  `json:"time"`
	RoomId     world.RoomId           `json:"roomId"`
	Bots       []*cao_world.Bot       `json:"bots"`
	Structures []*cao_world.Structure `json:"structures"`
	Resources  []*cao_world.Resource  `json:"resources"`
}

func NewGameStateHub() *GameStateHub {
	return &GameStateHub{
		Rooms:      map[world.RoomId]RoomState{},
		Clients:    map[*Client]bool{},
		WorldState: make(chan *cao_world.RoomEntities),
		Register:   make(chan *Client),
	}
}

func (hub *GameStateHub) Run() {
	for {
		select {
		case newEntities := <-hub.WorldState:
			time := newEntities.WorldTime
			rid := newEntities.GetRoomId()
			roomId := world.RoomId{
				Q: rid.Q,
				R: rid.R,
			}

			var state RoomState
			if s, ok := hub.Rooms[roomId]; ok {
				state = s
				state.Time = time
				state.RoomId = roomId
			} else {
				state = RoomState{
					Time:       time,
					RoomId:     roomId,
					Bots:       []*cao_world.Bot{},
					Structures: []*cao_world.Structure{},
					Resources:  []*cao_world.Resource{},
				}
			}
			state.Bots = newEntities.Bots
			state.Structures = newEntities.Structures
			state.Resources = newEntities.Resources

			hub.Rooms[roomId] = state

			pl, err := json.Marshal(state)
			if err != nil {
				log.Fatalf("failed to json marshal gamestate %v", err)
			}
			for client := range hub.Clients {
				if client.RoomId != roomId {
					continue
				}
				select {
				case client.Send <- pl:
				default:
					delete(hub.Clients, client)
					close(client.Send)
				}
			}
		case newClient := <-hub.Register:
			hub.Clients[newClient] = true
		case ex := <-hub.Unregister:
			if _, ok := hub.Clients[ex]; ok {
				delete(hub.Clients, ex)
			}
		}
	}
}
