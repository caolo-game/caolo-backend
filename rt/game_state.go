package world_state

type WorldState struct {
	GameConfig GameConfig `json:"gameConfig"`
}

type GameConfig struct {
	ExecutionLimit int `json:"execution_limit"`
}
