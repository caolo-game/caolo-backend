package ws

import (
	cao_world "github.com/caolo-game/cao-rt/cao_world_pb"

	"github.com/caolo-game/cao-rt/world"
)

type GameStateHub struct {
	Entities map[world.RoomId]RoomState
	Terrain  map[world.RoomId]*cao_world.RoomTerrain

	clients map[*client]bool

	// push new worldState to hub
	WorldState chan *cao_world.RoomEntities

	/// register new Clients
	register chan *client
	/// un-register new Clients
	unregister chan *client
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
		Entities:   map[world.RoomId]RoomState{},
		Terrain:    map[world.RoomId]*cao_world.RoomTerrain{},
		clients:    map[*client]bool{},
		WorldState: make(chan *cao_world.RoomEntities),
		register:   make(chan *client),
		unregister: make(chan *client),
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
			if s, ok := hub.Entities[roomId]; ok {
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

			hub.Entities[roomId] = state

			for client := range hub.clients {
				if client.roomId != roomId {
					continue
				}
				select {
				case client.entities <- &state:
				default:
					delete(hub.clients, client)
					close(client.entities)
				}
			}
		case newClient := <-hub.register:
			hub.clients[newClient] = true
		case ex := <-hub.unregister:
			if _, ok := hub.clients[ex]; ok {
				delete(hub.clients, ex)
			}
		}
	}
}
