import json
from dataclasses import dataclass
from typing import Dict, Callable, Optional, List
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
from ..queen import queen_channel


@dataclass
class GameState:
    world_time: int
    created: dt.datetime
    entities: Optional[Dict]
    rooms: List
    room_layout: List


async def load_initial_game_state(channel):
    stub = cao_world_pb2_grpc.WorldStub(channel)

    rooms = await stub.GetRoomList(cao_world_pb2.Empty())
    room_layout = await stub.GetRoomLayout(cao_world_pb2.Empty())

    return GameState(
        world_time=-1,
        created=dt.datetime.now(),
        entities=None,
        rooms=MessageToDict(rooms, including_default_value_fields=True)["roomIds"],
        room_layout=MessageToDict(room_layout, including_default_value_fields=True)[
            "positions"
        ],
    )


async def get_terrain(game_state: GameState, room_id: str):
    assert room_id in game_state.rooms
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

    async def _listener(self, queen_url: str):
        while 1:
            try:
                logging.info("Subscribing to world updates at %s", queen_url)
                # TODO: maybe use secure channel??
                channel = await queen_channel(queen_url)
                stub = cao_world_pb2_grpc.WorldStub(channel)

                self.game_state = await load_initial_game_state(channel)
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

                logging.warn("Queen stream ended. Retrying...")
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
        asyncio.create_task(self._listener(queen_url))


manager = GameStateManager()
