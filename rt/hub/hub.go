package hub

import (
	"log"

	cao_world "github.com/caolo-game/cao-rt/cao_world_pb"

	"github.com/caolo-game/cao-rt/client"
	"github.com/caolo-game/cao-rt/world"
)

type GameStateHub struct {
	Rooms   map[world.RoomId]RoomState
	Clients map[*client.Client]bool

	// push new worldState to hub
	WorldState chan *cao_world.RoomEntities
}

type RoomState struct {
	Time       int64
	RoomId     world.RoomId
	Bots       []*cao_world.Bot
	Structures []*cao_world.Structure
	Resources  []*cao_world.Resource
}

func NewRoomState() RoomState {
	return RoomState{
		Time:       -1,
		RoomId:     world.RoomId{},
		Bots:       []*cao_world.Bot{},
		Structures: []*cao_world.Structure{},
		Resources:  []*cao_world.Resource{},
	}
}

func NewGameStateHub() *GameStateHub {
	return &GameStateHub{
		Rooms:      map[world.RoomId]RoomState{},
		Clients:    map[*client.Client]bool{},
		WorldState: make(chan *cao_world.RoomEntities),
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
			} else {
				state = NewRoomState()
			}
			state.Time = time
			state.RoomId = roomId
			state.Bots = newEntities.Bots

			hub.Rooms[roomId] = state

			log.Printf("New room state boiiis time: %d room_id: %v", time, roomId)

		}
	}
}
