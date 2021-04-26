from typing import Dict, List, Tuple

from fastapi import APIRouter, Query, Depends, HTTPException, status

import grpc
import cao_world_pb2
import cao_world_pb2_grpc
import cao_common_pb2
from google.protobuf.json_format import MessageToDict

from ..queen import queen_channel


router = APIRouter(prefix="/world", tags=["world"])

TERRAIN_LAYOUT_CACHE = None


async def __get_room_terrain_layout():
    global TERRAIN_LAYOUT_CACHE
    if TERRAIN_LAYOUT_CACHE is not None:
        return TERRAIN_LAYOUT_CACHE
    channel = await queen_channel()
    stub = cao_world_pb2_grpc.WorldStub(channel)

    room_layout = await stub.GetRoomLayout(cao_world_pb2.Empty())
    TERRAIN_LAYOUT_CACHE = [(p.q, p.r) for p in room_layout.positions]

    return TERRAIN_LAYOUT_CACHE


@router.get("/room-terrain-layout", response_model=List[Tuple[int, int]])
async def room_terrain_layout():
    """
    return the coordinates of the room grid points in a list.

    If you query the terrain the i-th terrain enum value
    will correspond to the i-th coordinates returned by this endpoint
    """
    return await __get_room_terrain_layout()


@router.get("/tile-enum")
async def tile_enum_values():
    """
    The dictionary returned by this endpoint can be used to map Terrain enum values to string values if necessary.
    """
    return {x.index: str(x.name) for x in cao_world_pb2._TERRAIN.values}


@router.get("/rooms", response_model=List[Dict])
async def rooms():
    channel = await queen_channel()
    stub = cao_world_pb2_grpc.WorldStub(channel)

    res = await stub.GetRoomList(cao_world_pb2.Empty())

    return MessageToDict(
        res, including_default_value_fields=True, preserving_proto_field_name=False
    ).get("roomIds", [])
