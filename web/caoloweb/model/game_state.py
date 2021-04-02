import json
from dataclasses import dataclass
from typing import Dict, Callable
import datetime as dt
import asyncio
import logging

from aioredis import Redis

from ..config import QUEEN_TAG
from ..api_schema import RoomObjects


@dataclass
class GameState:
    world_time: int
    created: dt.datetime
    payload: Dict


def get_terrain(game_state: GameState, room_id: str):
    terrain = game_state.payload["terrain"].get("roomTerrain", {})
    return terrain.get(room_id, [])


def get_room_objects(game_state: GameState, room_id: str):
    payload = RoomObjects()
    payload.time = game_state.world_time
    payload.payload = {
        "bots": game_state.payload["bots"].get(room_id, []),
        "structures": game_state.payload["structures"].get(room_id, []),
        "resources": game_state.payload["resources"].get(room_id, []),
    }
    return payload


async def load_latest_game_state(db, queen_tag=None) -> GameState:
    if queen_tag is None:

        queen_tag = QUEEN_TAG

    row = await db.fetchrow(
        """
        SELECT
            t.payload as payload
            , t.world_time as world_time
            , t.created as created
        FROM public.world_output t
        WHERE t.queen_tag=$1
        ORDER BY t.created DESC
        """,
        queen_tag,
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

    async def _load_from_db(self, db):
        self.game_state = await load_latest_game_state(db)

    async def _listener(self, queen_tag: str, redis: Redis):
        key = f"{queen_tag}-world"
        while 1:
            logging.info("Subscribing to %s", key)
            ch = await redis.subscribe(key)
            ch = ch[0]

            async for msg in ch.iter(decoder=json.loads):
                logging.info("Got a new game state of tick %d" % msg["time"])
                self.game_state = GameState(
                    world_time=msg["time"],
                    created=dt.datetime.now(),
                    payload=msg,
                )
                for cb in self.on_new_state_callbacks:
                    try:
                        cb(self.game_state)
                    except:
                        logging.exception("Callback failed")
            logging.warn(
                "GameStateManager._listener exiting. channel: %s.",
                ch,
            )

    async def start(self, queen_tag: str, redis: Redis, db):
        await self._load_from_db(db)
        asyncio.create_task(self._listener(queen_tag, redis))


manager = GameStateManager()
