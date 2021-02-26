import json
from dataclasses import dataclass
from typing import Dict, Callable
import datetime as dt
import asyncio
import logging

from ..api_schema import RoomObjects


@dataclass
class GameState:
    world_time: int
    created: dt.datetime
    payload: Dict


def get_room_objects(game_state: GameState, room_id: str):
    payload = RoomObjects()
    payload.time = game_state.world_time
    payload.payload = {
        "bots": game_state.payload["bots"].get(room_id, []),
        "structures": game_state.payload["structures"].get(room_id, []),
        "resources": game_state.payload["resources"].get(room_id, []),
    }
    return payload


async def load_latest_game_state(db) -> GameState:
    row = await db.fetchrow(
        """
        SELECT
            t.payload as payload
            , t.world_time as world_time
            , t.created as created
        FROM public.world_output t
        ORDER BY t.created DESC
        """,
    )
    return GameState(
        world_time=row["world_time"],
        created=row["created"],
        payload=json.loads(row["payload"]),
    )


class GameStateManager:
    def __init__(self):
        self.game_state = None
        self.running = False
        self.on_new_state_callbacks = []
        self.sleep_time = 0

    def on_new_state(self, func: Callable[[GameState], None]):
        self.on_new_state_callbacks.append(func)

    def deregister_cb(self, func):
        try:
            self.on_new_state_callbacks.remove(func)
        except ValueError:
            pass

    async def load_next_state(self):
        state = self.game_state
        # figure out how much we need to sleep
        # formula: expected_next - now
        # expected_next = end of last tick + target latency + actual latency for bias
        try:
            end = state.payload["diagnostics"]["tick_end"]
            end = dt.datetime.strptime(end[:-4], "%Y-%m-%dT%H:%M:%S.%f")
        except Exception as err:
            logging.warn(f"Failed to parse end timestamp {err}")
            end = dt.datetime.now()
        latency = state.payload["diagnostics"]["tick_latency_ms"]
        target_lat = state.payload["gameConfig"]["target_tick_ms"]
        expected_next = end + dt.timedelta(milliseconds=target_lat + latency)
        now = dt.datetime.utcnow()
        delta = expected_next - now
        self.sleep_time = delta

    async def run(self, pool):
        """
        :param pool: database connection pool
        """
        assert not self.running
        self.running = True
        try:
            while 1:
                yield
                async with pool.acquire() as con:
                    state = await load_latest_game_state(con)
                    if not self.game_state or state.created != self.game_state.created:
                        self.game_state = state
                        for cb in self.on_new_state_callbacks:
                            cb(state)
                logging.debug(f"sleeping for {self.sleep_time} seconds")
                await asyncio.sleep(max(self.sleep_time, 0.01))
        finally:
            self.running = False


manager = GameStateManager()
