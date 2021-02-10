package game_state

import (
	"encoding/json"
	"github.com/jmoiron/sqlx"
)

type GameState struct {
	Bots           map[string]interface{} `json:"bots"`
	Structures     map[string]interface{} `json:"structures"`
	Resources      map[string]interface{} `json:"resources"`
	Rooms          map[string]interface{} `json:"rooms"`
	Users          interface{}            `json:"users"`
	Terrain        interface{}            `json:"terrain"`
	RoomProperties interface{}            `json:"roomProperties"`
	GameConfig     GameConfig             `json:"gameConfig"`
	Time           int64
}

type GameConfig struct {
	ExecutionLimit int `json:"execution_limit"`
}

var gameStateQuery = `
SELECT t.payload, t.world_time
FROM world_output t
ORDER BY t.created DESC
LIMIT 1
`

func GetLatestGameState(db *sqlx.DB) (*GameState, error) {

	type GameQResult struct {
		Payload []byte `db:"payload"`
		Time    int64  `db:"world_time"`
	}

	results := []GameQResult{}
	err := db.Select(&results, gameStateQuery)
	if err != nil {
		return nil, err
	}
	var state GameState
	err = json.Unmarshal(results[0].Payload, &state)
	if err != nil {
		return nil, err
	}
	state.Time = results[0].Time
	return &state, nil
}
