import json
from dataclasses import dataclass
from typing import Dict, Callable
import datetime as dt
import asyncio
import logging

from ..api_schema import RoomObjects

import aioredis
from aioredis import Redis


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
        self.on_new_state_callbacks = []

    def on_new_state(self, func: Callable[[GameState], None]):
        self.on_new_state_callbacks.append(func)

    def deregister_cb(self, func):
        try:
            self.on_new_state_callbacks.remove(func)
        except ValueError:
            pass

    async def _listener(self, ch):
        while await ch.wait_message():
            msg = await ch.get_json()
            self.game_state = GameState(
                world_time=msg["time"], created=dt.datetime.now(), payload=msg
            )
            for cb in self.on_new_state_callbacks:
                try:
                    cb(self.game_state)
                except:
                    logging.exception("Callback failed")

    async def start(self, queen_tag: str, redis: Redis):
        ch = await redis.subscribe(f"{queen_tag}-world")
        ch = ch[0]
        asyncio.create_task(self._listener(ch))


manager = GameStateManager()
