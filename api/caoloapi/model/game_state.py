import json
from dataclasses import dataclass
from typing import Dict, Callable, Optional
import datetime as dt
import asyncio
import logging

from aioredis import Redis
import grpc
from google.protobuf.json_format import MessageToDict

import cao_world_pb2
import cao_world_pb2_grpc

from ..config import QUEEN_TAG
from ..api_schema import RoomObjects, make_room_id


@dataclass
class GameState:
    world_time: int
    created: dt.datetime
    entities: Optional[Dict]
    properties: Optional[Dict]


def get_terrain(game_state: GameState, room_id: str):
    assert game_state.properties, "Load properties before getting room_objects"
    terrain = game_state.properties["terrain"].get("roomTerrain", {})
    return terrain.get(room_id, [])


def __to_dict(obj):
    if obj is not None:
        return MessageToDict(obj, preserving_proto_field_name=False)
    return {}


def get_room_objects(game_state: GameState, room_id: str):
    assert game_state.entities, "Load entities before getting room_objects"

    payload = RoomObjects()
    payload.time = game_state.world_time
    payload.payload = {
        "bots": __to_dict(game_state.entities["bots"].get(room_id)).get("bots", []),
        "structures": __to_dict(game_state.entities["structures"].get(room_id)).get(
            "structures", []
        ),
        "resources": __to_dict(game_state.entities["resources"].get(room_id)).get(
            "resources", []
        ),
    }
    return payload


async def load_world_constants(db, queen_tag=None) -> GameState:
    if queen_tag is None:
        queen_tag = QUEEN_TAG

    props = await db.fetchrow(
        """
    SELECT
        t.payload
    FROM public.world_const t
    WHERE t.queen_tag=$1
    """,
        queen_tag,
    )

    if props:
        return GameState(
            world_time=-1,
            created=None,
            entities=None,
            properties=json.loads(props["payload"]),
        )
    return None


class GameStateManager:
    def __init__(self):
        self.game_state: Optional[GameState] = None
        self.on_new_state_callbacks = []

    def on_new_state(self, func: Callable[[GameState], None]):
        self.on_new_state_callbacks.append(func)

    def deregister_cb(self, func):
        try:
            self.on_new_state_callbacks.remove(func)
        except ValueError:
            pass

    async def _load_from_db(self, db, queen_tag=None):
        if self.game_state:
            new_state = await load_latest_game_state_after(
                db, self.game_state.world_time, queen_tag
            )
        else:
            new_state = await load_world_constants(db, queen_tag)
        if new_state:
            self.game_state = new_state
            logging.debug(
                "Loaded game-state # %d of tag %s",
                self.game_state.world_time,
                queen_tag,
            )
        else:
            logging.debug("No new state is available")

    async def _listener(self, queen_url: str):
        while 1:
            try:
                logging.info("Subscribing to world updates at %s", queen_url)
                # TODO: maybe use secure channel??
                channel = grpc.aio.insecure_channel(queen_url)
                stub = cao_world_pb2_grpc.WorldStub(channel)

                async for msg in stub.Entities(cao_world_pb2.Empty()):
                    payload = {
                        "bots": {},
                        "resources": {},
                        "structures": {},
                        "diagnostics": None,
                    }
                    for room_bots in msg.bots:
                        room_id = make_room_id(room_bots.roomId.q, room_bots.roomId.r)
                        payload["bots"][room_id] = room_bots
                    for room_resources in msg.resources:
                        room_id = make_room_id(
                            room_resources.roomId.q, room_resources.roomId.r
                        )
                        payload["resources"][room_id] = room_resources
                    for room_structures in msg.structures:
                        room_id = make_room_id(
                            room_structures.roomId.q, room_structures.roomId.r
                        )
                        payload["structures"][room_id] = room_structures

                    if msg.diagnostics:
                        payload["diagnostics"] = MessageToDict(
                            msg.diagnostics, preserving_proto_field_name=False
                        )

                    assert self.game_state is not None
                    self.game_state.world_time = msg.worldTime
                    self.game_state.created = dt.datetime.now()
                    self.game_state.entities = payload

                    for cb in self.on_new_state_callbacks:
                        try:
                            cb(self.game_state)
                        except:
                            logging.exception("Callback failed")

                logging.warn("GameStateManager._listener exiting")
                return
            except grpc.aio.AioRpcError as err:
                if err.code() in (grpc.StatusCode.UNAVAILABLE, grpc.StatusCode.UNKNOWN):
                    logging.warn("Cao-Queen is unavailable. Retrying...")
                else:
                    logging.exception("gRPC error in GameState listender")
                    raise
            except:
                logging.exception("Error in GameState listener")
                raise

    async def start(self, queen_tag: str, queen_url: str, db):
        await self._load_from_db(db, queen_tag)
        asyncio.create_task(self._listener(queen_url))


manager = GameStateManager()
