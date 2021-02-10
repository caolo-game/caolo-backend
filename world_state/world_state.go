package world_state

import (
	"encoding/json"
	"github.com/jmoiron/sqlx"
)

type WorldState struct {
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

var WorldStateQuery = `
SELECT t.payload, t.world_time
FROM world_output t
ORDER BY t.created DESC
LIMIT 1
`

func GetLatestWorldState(db *sqlx.DB) (*WorldState, error) {

	type WorldQResult struct {
		Payload []byte `db:"payload"`
		Time    int64  `db:"world_time"`
	}

	results := []WorldQResult{}
	err := db.Select(&results, WorldStateQuery)
	if err != nil {
		return nil, err
	}
	var state WorldState
	err = json.Unmarshal(results[0].Payload, &state)
	if err != nil {
		return nil, err
	}
	state.Time = results[0].Time
	return &state, nil
}
